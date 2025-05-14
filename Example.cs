using System.Runtime.InteropServices;
using Wasmtime;

public class WasmClient : IDisposable
{
    private readonly Store store;
    private readonly Instance instance;

    public WasmClient(string wasmPath)
    {
        using var engine = new Engine();
        using var module = Module.FromFile(engine, wasmPath);
        store = new Store(engine);
        instance = new Instance(store, module, Array.Empty<Extern>());
    }

    public string GetString(string input)
    {
        var getString = instance.GetFunction(store, "get_string")
            ?? throw new Exception("get_string function not found");

        // Convert input string to null-terminated UTF-8
        var inputBytes = System.Text.Encoding.UTF8.GetBytes(input + "\0");
        var inputPtr = Marshal.AllocHGlobal(inputBytes.Length);
        try
        {
            Marshal.Copy(inputBytes, 0, inputPtr, inputBytes.Length);
            
            // Call the WebAssembly function
            var resultPtr = (IntPtr)getString.Invoke(store, inputPtr);
            if (resultPtr == IntPtr.Zero)
            {
                return string.Empty;
            }

            // Convert result back to string
            return Marshal.PtrToStringUTF8(resultPtr) ?? string.Empty;
        }
        finally
        {
            Marshal.FreeHGlobal(inputPtr);
        }
    }

    public void Dispose()
    {
        store?.Dispose();
    }
} 