/* tslint:disable */
/* eslint-disable */

export class KeyStorage {
  free(): void;
  [Symbol.dispose](): void;
  constructor(db_name: string);
  /**
   * Сохраняет приватные ключи
   */
  save_private_keys(_user_id: string, _identity_private: Uint8Array, _signed_prekey_private: Uint8Array, _verifying_private: Uint8Array): Promise<void>;
  /**
   * Загружает приватные ключи
   */
  load_private_keys(_user_id: string): Promise<any>;
  /**
   * Сохраняет сессию Double Ratchet
   */
  save_session(_session_id: string, _contact_id: string, _session_data: Uint8Array): Promise<void>;
}

export class MessengerAPI {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
}

/**
 * Создать нового криптографического клиента
 */
export function create_crypto_client(): string;

/**
 * Расшифровать сообщение
 * encrypted_json - JSON строка с зашифрованным сообщением
 * Возвращает расшифрованный текст
 */
export function decrypt_message(client_id: string, session_id: string, encrypted_json: string): string;

/**
 * Удалить клиента из памяти
 */
export function destroy_client(client_id: string): void;

/**
 * Зашифровать сообщение
 * Возвращает JSON с зашифрованным сообщением
 */
export function encrypt_message(client_id: string, session_id: string, plaintext: string): string;

/**
 * Получить публичные ключи клиента для регистрации (JSON)
 */
export function get_registration_bundle(client_id: string): string;

export function init(): void;

/**
 * Инициализировать сессию получателя при получении первого сообщения
 * first_message_json - JSON строка с первым зашифрованным сообщением от отправителя
 * Возвращает session_id
 */
export function init_receiving_session(client_id: string, contact_id: string, remote_bundle_json: string, first_message_json: string): string;

/**
 * Инициализировать сессию с контактом (отправитель)
 * remote_bundle_json - JSON строка с ключами удаленной стороны
 * Возвращает session_id
 */
export function init_session(client_id: string, contact_id: string, remote_bundle_json: string): string;

export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_messengerapi_free: (a: number, b: number) => void;
  readonly __wbg_keystorage_free: (a: number, b: number) => void;
  readonly keystorage_new: (a: number, b: number) => number;
  readonly keystorage_save_private_keys: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => any;
  readonly keystorage_load_private_keys: (a: number, b: number, c: number) => any;
  readonly keystorage_save_session: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => any;
  readonly create_crypto_client: () => [number, number, number, number];
  readonly get_registration_bundle: (a: number, b: number) => [number, number, number, number];
  readonly init_session: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
  readonly init_receiving_session: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
  readonly encrypt_message: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
  readonly decrypt_message: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
  readonly destroy_client: (a: number, b: number) => [number, number];
  readonly init: () => void;
  readonly version: () => [number, number];
  readonly wasm_bindgen__convert__closures_____invoke__h712b99b0a85efce4: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__heb50c19e479af15c: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h0211784e36f3c976: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
