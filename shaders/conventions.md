## Bindgroups and bindings

Since bindgroups are limited to 4 ([by default](https://www.w3.org/TR/webgpu/#limits), `maxBindGroups`) and must be consecutive and start at 0, while `maxBindingsPerBindGroup` is at 1000 and can be non-consecutive (TODO test), I will organize bindings based on that.

Currently I will provide each 'different concern' 20 bindings.
Its not as if its difficult to change if that doesn't suffice, its just annoying.

So
* Shader exklusive settings and resolution get [0,20)
* multimodal gaussian gets [20,40)
* next binary-distance-approx (or random walk montecarlo hastings, TBD what nature this will have)

## Filenaming

Any file may contain bindings, they should be placed next to the usage, and can be differentiated with [above](#bindgroups-and-bindings).

If the file contains an entrypoint it will have a name for that, so `.fragment.wgsl` contains a fragment.
