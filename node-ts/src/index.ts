import { captureEvent } from "./wasm";

const host = process.argv[2] || "http://localhost:8000";
const apiKey =
  process.argv[3] || "phc_3P4u1dFCYoee5GXN40fecOKniv1HbmOPj9lZovECY8z";

try {
  new URL(host); // Validate URL
} catch (e) {
  console.error("Invalid URL. Please provide a valid URL.");
  process.exit(1);
}

const result = await captureEvent(
  "$pageview",
  "distinct_id_123",
  apiKey,
  {
    $current_url: "/tulum-hackathon",
    plan: "pro",
    paid: "you know it!",
  },
  host
);
