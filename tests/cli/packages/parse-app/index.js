import { parseAs } from "@kei/hono";

function isUserRequestShape(value) {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof value.name === "string"
  );
}

export function parseUserRequest(text) {
  return parseAs(text, isUserRequestShape);
}
