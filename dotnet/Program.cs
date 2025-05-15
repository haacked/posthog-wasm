using PostHog;

var host = args.Length > 0
    ? args[0]
    : "http://localhost:8000";

var apiKey = args.Length > 1
    ? args[1]
    : "phc_jtUhKM2jBb9bN31USuNxqs2IiR2w43EgqC6AY4iaWVo";

if (!Uri.TryCreate(host, UriKind.Absolute, out var hostUri))
{
    Console.WriteLine("Invalid URL. Please provide a valid URL.");
    return;
}

var wasmClient = new WasmClient(hostUri);
var result = await wasmClient.CaptureAsync(
    eventName: "test_event",
    distinctId: "distinct_id_123",
    apiKey: apiKey,
    properties: new()
    {
        ["plan"] = "pro",
        ["paid"] = "you know it!"
    });
Console.WriteLine("Response Body:\n" + result);