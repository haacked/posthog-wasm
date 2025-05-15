using PostHog;

var wasmClient = new WasmClient();
var result = await wasmClient.CaptureAsync(
    eventName: "test_event",
    distinctId: "8675309",
    apiKey: "phc_jtUhKM2jBb9bN31USuNxqs2IiR2w43EgqC6AY4iaWVo",
    properties: new()
    {
        ["plan"] = "pro",
        ["paid"] = "you know it!"
    });
Console.WriteLine(result);