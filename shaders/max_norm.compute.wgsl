#import "binary_distance_approx.buffer.wgsl";

@group(1) @binding(41)
var<storage, read> unnormalized: array<u32>;

@group(1) @binding(42)
var<storage, read_write> normalized: array<f32>;

// apparently initialized to 0:
// https://github.com/gpuweb/cts/issues/1487
@group(1) @binding(43)
// this ought to be legal according to spec:
// > Atomic types may only be instantiated by variables in the workgroup address space or by storage buffer variables with a read_write access mode.
// But this either errors on "missing binding for a storage buffer" or on "unsupported type for binding fields"
var<storage, read_write> maximum: atomic<u32>;
var<workgroup> workgroup_maximum: atomic<u32>;

@compute
// sqrt(256) = 16
// 256 is default limit for total workgroup size
@workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let is_leader = all(local_id == vec3(0));
    let idx = to_buffer_idx(global_id.xy);
    atomicMax(&workgroup_maximum, unnormalized[idx]);
    workgroupBarrier();
    if (is_leader) {
        atomicMax(&maximum, atomicLoad(&workgroup_maximum));
    }
    // TODO: huh, this looks suspiciously like a global barrier.
    // I must be missing something?
    // Yeah. Its highly misleading IMO.
    // https://github.com/gpuweb/gpuweb/issues/3774
    // Should not work.
    storageBarrier();
    let maximum = f32(atomicLoad(&maximum));
    normalized[idx] = f32(unnormalized[idx]) / maximum;
}
