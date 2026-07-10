import type { Option } from "@kei/runtime";

export interface UserRequestShape {
  readonly name: string;
}

export declare function parseUserRequest(text: string): Option<UserRequestShape>;
