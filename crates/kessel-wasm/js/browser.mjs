import init, * as bindings from '../pkg/web/kessel_wasm.js';
import { WasmSimulator } from './adapter.mjs';

/** Supply raw pool and/or catalogue JSON text; no filesystem or Node APIs are used. */
export async function createSimulator({ poolJson, catalogueJson, wasm } = {}) {
    await init(wasm);
    return new WasmSimulator(bindings, poolJson, { catalogueJson });
}
