using System.Net.Http.Headers;
using System.Text;
using PostHog.Json;
using Wasmtime;

namespace PostHog;

public class WasmClient
{
    readonly Instance _instance;
    readonly Func<int, int> _alloc;
    readonly Memory _memory;
    readonly Action<int, int> _dealloc;
    int _latestHttpRequestResponseLength;

    public WasmClient()
    {
        var engine = new Engine();
        var module = Module.FromFile(engine, "posthog_wasm.wasm");
        var linker = new Linker(engine);
        var store = new Store(engine);

        // Define http_request and http_request_len imports
        linker.Define("env", "http_request", Function.FromCallback<int, int, int, int, int, int, int>(store,
            (urlPtr, urlLen, methodPtr, methodLen, bodyPtr, bodyLen) =>
            {
                var allocFunc = _alloc ?? throw new InvalidOperationException("alloc function is null");
                var memory = _memory ?? throw new InvalidOperationException("memory is null");
                var url = memory.ReadString(urlPtr, (uint)urlLen); // Use local 'memory'
                var method = memory.ReadString(methodPtr, (uint)methodLen); // Use local 'memory'

                using var httpClient = new HttpClient();
                var httpMethod = new HttpMethod(method.Trim().ToUpperInvariant());
                var requestMessage = new HttpRequestMessage(httpMethod, url);

                if (bodyLen > 0)
                {
                    var requestBodyBytes = new byte[bodyLen];
                    memory.ReadBytes(bodyPtr, requestBodyBytes); // Use local 'memory'

                    var requestBodyJson = JsonSerializerHelper.FormatJson(Encoding.UTF8.GetString(requestBodyBytes));
                    Console.WriteLine("Request Body JSON: \n" + requestBodyJson);

                    requestMessage.Content = new ByteArrayContent(requestBodyBytes);
                    requestMessage.Content.Headers.ContentType = new MediaTypeHeaderValue("application/json");
                }
                else if (httpMethod == HttpMethod.Post || httpMethod == HttpMethod.Put || httpMethod == HttpMethod.Patch)
                {
                    requestMessage.Content = new StringContent("{}", Encoding.UTF8, "application/json");
                }

                var response = httpClient.SendAsync(requestMessage).Result; // ✅ blocking
                response.EnsureSuccessStatusCode();
                var responseBody = response.Content.ReadAsStringAsync().Result;
                var responseBodyBytes = Encoding.UTF8.GetBytes(responseBody);

                // 'allocFunc' (local variable) will be assigned after instantiation but captured by the lambda.
                int responseBodyPtrInWasm = allocFunc(responseBodyBytes.Length);
                memory.WriteBytes(responseBodyPtrInWasm, responseBodyBytes); // Use local 'memory'

                _latestHttpRequestResponseLength = responseBodyBytes.Length; // Access field
                return responseBodyPtrInWasm;
            }
        ));

        linker.Define("env", "http_request_len", Function.FromCallback(store,
            () => _latestHttpRequestResponseLength // Access field
        ));

        linker.Define(
            "env", "log_message",
            Function.FromCallback<int, int>(store, (int messagePtr, int messageLen) =>
            {
                var memory = _memory ?? throw new InvalidOperationException("memory is null");
                var message = memory.ReadString(messagePtr, (uint)messageLen);
                Console.WriteLine($"WASM log: {message}");
            })
        );

        // 👇 Now instantiate
        _instance = linker.Instantiate(store, module);

        // 👇 Set fields after instantiation
        // The local 'memory' and 'allocFunc' variables used in the lambdas get their values here.
        _memory = _instance.GetMemory("memory")!;

        _alloc = _instance.GetFunction<int, int>("alloc_buffer")
                 ?? throw new InvalidOperationException("alloc function not found");

        var dealloc = _instance.GetFunction("dealloc_buffer")
            ?? throw new InvalidOperationException("dealloc function not found");
        _dealloc = dealloc.WrapAction<int, int>()
            ?? throw new InvalidOperationException("dealloc function not wrapped");
    }
    
    public async Task<string> CaptureAsync(
        string eventName,
        string distinctId,
        string apiKey,
        Dictionary<string, object> properties)
    {
        var (eventBytes, eventPtr) = WriteString(eventName);
        var (distinctIdBytes, distinctIdPtr) = WriteString(distinctId);
        var (apiKeyBytes, apiKeyPtr) = WriteString(apiKey);
        var propertiesJson = await JsonSerializerHelper.SerializeToCamelCaseJsonStringAsync(properties, writeIndented: true);
        var (propertiesBytes, propertiesPtr) = WriteString(propertiesJson);

        var captureFunc = _instance.GetFunction<int, int, int, int, int, int, int, int, int>("capture")
            ?? throw new InvalidOperationException("Wasm 'capture' function not found.");

        int resultPtr = captureFunc(
            eventPtr,
            eventBytes.Length,
            distinctIdPtr,
            distinctIdBytes.Length,
            apiKeyPtr,
            apiKeyBytes.Length,
            propertiesPtr,
            propertiesBytes.Length);

        int responseLen = _latestHttpRequestResponseLength; // This is a guess, depends on http_post_len behavior
        if (responseLen == 0) responseLen = 2048; // Fallback to original risky assumption

        string result = _memory.ReadString(resultPtr, (uint)responseLen);
        _dealloc(eventPtr, eventBytes.Length);
        _dealloc(distinctIdPtr, distinctIdBytes.Length);
        _dealloc(apiKeyPtr, apiKeyBytes.Length);
        _dealloc(resultPtr, responseLen); // Deallocating the result from capture
        return result;
    }

    (byte[] bytes, int pointer) WriteString(string s)
    {
        var bytes = Encoding.UTF8.GetBytes(s);
        var ptr = _alloc(bytes.Length);
        _memory.WriteBytes(ptr, bytes);
        return (bytes, ptr);
    }
}

public static class WasmMemoryExtensions
{
    public static void WriteBytes(this Memory memory, int offset, byte[] data)
    {
        // Check against total memory length
        if (offset < 0 || (long)offset + data.Length > memory.GetLength()) // Added check for negative offset
            throw new ArgumentOutOfRangeException(nameof(offset), $"Attempted to write outside WASM memory bounds. Offset: {offset}, Data Length: {data.Length}, Memory Size: {memory.GetLength()}");

        // Get the specific span for writing
        var destination = memory.GetSpan(offset, data.Length);
        data.CopyTo(destination);
    }

    public static string ReadString(this Memory memory, int offset, uint length)
    {
        // Check against total memory length before reading
        if (offset < 0 || offset + length > memory.GetLength()) // Added check for negative offset
            throw new ArgumentOutOfRangeException(nameof(offset), $"Attempted to read outside WASM memory bounds. Offset: {offset}, Length: {length}, Memory Size: {memory.GetLength()}");

        var span = memory.GetSpan(offset, (int)length); // This was already correct
        return Encoding.UTF8.GetString(span);
    }

    // Overload for ReadBytes used in http_request callback
    public static void ReadBytes(this Memory memory, int offset, byte[] buffer)
    {
        // Check against total memory length
        if (offset < 0 || (long)offset + buffer.Length > memory.GetLength()) // Added check for negative offset
            throw new ArgumentOutOfRangeException(nameof(offset), $"Attempted to read outside WASM memory bounds. Offset: {offset}, Buffer Length: {buffer.Length}, Memory Size: {memory.GetLength()}");

        // Get the specific span for reading
        var source = memory.GetSpan(offset, buffer.Length);
        source.CopyTo(buffer);
    }
}
