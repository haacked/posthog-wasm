// WebAssembly memory management
let memory: WebAssembly.Memory;
// define all our `pub extern` functions
let alloc: (size: number) => number;
let dealloc: (ptr: number, size: number) => void;
let capture: (
  event_name_ptr: number,
  event_name_len: number,
  distinct_id_ptr: number,
  distinct_id_len: number,
  api_key_ptr: number,
  api_key_len: number,
  properties_ptr: number,
  properties_len: number,
  host_ptr: number,
  host_len: number
) => number;

// Store the latest response length.
// we need something better for concurrency
// maybe a map of request id?
let latestHttpRequestResponseLength = 0;

// Store the current host data
let currentHostData: { ptr: number; len: number } | null = null;

// yes, I know. I just want to avoid using classes.
let isInitialized = false;

// Initialize WebAssembly module
const initWasm = async () => {
  // bail if we already initialized the WASM instance.
  if (isInitialized) {
    return;
  }

  // Load the wasm file. This is Bun specific, sorry.
  const wasmModule = await Bun.file("posthog_wasm.wasm").arrayBuffer();

  // Instantiate the wasm module
  const { instance } = await WebAssembly.instantiate(wasmModule, {
    env: {
      memory: new WebAssembly.Memory({ initial: 256 }),
      http_request: async (
        url_ptr: number,
        url_len: number,
        method_ptr: number,
        method_len: number,
        body_ptr: number,
        body_len: number
      ): Promise<number> => {
        // get the url and method from wasm memory
        const path = wasmToString(url_ptr, url_len);
        const method = wasmToString(method_ptr, method_len)
          .trim()
          .toUpperCase();

        // Get the host from the stored host data
        if (!currentHostData) {
          throw new Error(
            "Host not set. Make sure to call captureEvent first."
          );
        }
        const host = wasmToString(currentHostData.ptr, currentHostData.len);
        const url = new URL(path, host).toString();

        // Prepare request options
        const options: RequestInit = {
          method,
          headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
          },
        };

        // Add body if present
        if (body_len > 0) {
          const bodyBytes = new Uint8Array(memory.buffer).slice(
            body_ptr,
            body_ptr + body_len
          );
          options.body = bodyBytes;

          console.log(
            `Request Body JSON:\n${JSON.stringify(
              JSON.parse(wasmToString(body_ptr, body_len)),
              null,
              2
            )}`
          );
        } else if (["POST", "PUT", "PATCH"].includes(method)) {
          options.body = "{}";
          console.log(`Request Body JSON:\n{}`);
        }

        // make the request
        const response = await fetch(url, options);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        // get the response body as JSON
        const responseBody = await response.json();
        const responseBodyStr = JSON.stringify(responseBody);
        const responseBodyBytes = new TextEncoder().encode(responseBodyStr);

        // Allocate memory in WebAssembly and write response
        const responseBodyPtr = alloc(responseBodyBytes.length);
        new Uint8Array(memory.buffer).set(responseBodyBytes, responseBodyPtr);

        // Store response length for http_request_len
        latestHttpRequestResponseLength = responseBodyBytes.length;

        console.log(
          `Response Body JSON:\n${JSON.stringify(
            JSON.parse(responseBodyStr),
            null,
            2
          )}`
        );

        return responseBodyPtr;
      },
      http_request_len: (): number => {
        return latestHttpRequestResponseLength;
      },
      log_message: (message_ptr: number, message_len: number) => {
        const message = wasmToString(message_ptr, message_len);
        console.log(
          `WASM log: ${JSON.stringify(JSON.parse(message), null, 2)}`
        );
      },
    },
  });

  // Initialize memory and function pointers
  const exports = instance.exports;
  memory = exports.memory as WebAssembly.Memory;
  alloc = exports.alloc_buffer as (size: number) => number;
  dealloc = exports.dealloc_buffer as (ptr: number, size: number) => void;
  capture = exports.capture as (
    event_name_ptr: number,
    event_name_len: number,
    distinct_id_ptr: number,
    distinct_id_len: number,
    api_key_ptr: number,
    api_key_len: number,
    properties_ptr: number,
    properties_len: number,
    host_ptr: number,
    host_len: number
  ) => number;

  isInitialized = true;
};

// convert a string to wasm memory
const stringToWasm = (str: string) => {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(str);
  const ptr = alloc(bytes.length);
  new Uint8Array(memory.buffer).set(bytes, ptr);
  return { ptr, len: bytes.length };
};

// get a string from wasm memory
const wasmToString = (ptr: number, len: number) => {
  const bytes = new Uint8Array(memory.buffer).slice(ptr, ptr + len);
  return new TextDecoder().decode(bytes);
};

export const captureEvent = async (
  eventName: string,
  distinctId: string,
  apiKey: string,
  properties: Record<string, any> = {},
  host: string = "http://localhost:8000"
) => {
  // Initialize the wasm module
  await initWasm();

  // Convert all parameters to WebAssembly memory
  const eventNameData = stringToWasm(eventName);
  const distinctIdData = stringToWasm(distinctId);
  const apiKeyData = stringToWasm(apiKey);
  const propertiesData = stringToWasm(JSON.stringify(properties));
  const hostData = stringToWasm(host);

  // Store the host data for use in http_request
  if (currentHostData) {
    dealloc(currentHostData.ptr, currentHostData.len);
  }
  currentHostData = hostData;

  // Call the capture function with all required parameters
  const respPtr = capture(
    eventNameData.ptr,
    eventNameData.len,
    distinctIdData.ptr,
    distinctIdData.len,
    apiKeyData.ptr,
    apiKeyData.len,
    propertiesData.ptr,
    propertiesData.len,
    hostData.ptr,
    hostData.len
  );

  // Get the response length from http_request_len
  const respLen = latestHttpRequestResponseLength;

  // Convert the response to a string
  const result = wasmToString(respPtr, respLen);

  // Deallocate all memory except host data (which we keep for future requests)
  dealloc(eventNameData.ptr, eventNameData.len);
  dealloc(distinctIdData.ptr, distinctIdData.len);
  dealloc(apiKeyData.ptr, apiKeyData.len);
  dealloc(propertiesData.ptr, propertiesData.len);
  dealloc(respPtr, respLen);

  return result;
};
