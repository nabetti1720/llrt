import { Performance } from "perf_hooks";

export {};

declare global {
  namespace QuickJS {
    type TypedArray =
      | Uint8Array
      | Uint8ClampedArray
      | Uint16Array
      | Uint32Array
      | Int8Array
      | Int16Array
      | Int32Array
      | BigUint64Array
      | BigInt64Array
      | Float32Array
      | Float64Array;
    type ArrayBufferView = TypedArray | DataView;
  }

  var performance: Performance;
}
