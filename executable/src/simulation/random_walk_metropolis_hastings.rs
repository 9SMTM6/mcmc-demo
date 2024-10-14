use macros::{cfg_educe_debug, cfg_persistence_derive};

use crate::target_distributions::multimodal_gaussian::GaussianTargetDistr;

use crate::visualizations::shader_based::diff_display::shader_bindings::RWMHAcceptRecord;

use super::{Percentage, RngIter, StandardNormal};

#[cfg_persistence_derive]
#[derive(Clone)]
#[cfg_educe_debug]
pub struct GaussianProposal {
    pub sigma: f32,
    pub rng: RngIter<StandardNormal>,
}

impl Default for GaussianProposal {
    fn default() -> Self {
        Self {
            sigma: 0.2,
            rng: Default::default(),
        }
    }
}

#[cfg_persistence_derive]
#[derive(Clone)]
#[cfg_educe_debug]
pub struct IPromiseThisIsNonZeroUsize(usize);

impl IPromiseThisIsNonZeroUsize {
    pub const fn new(val: usize) -> Self {
        if val == 0 {
            panic!("nonzero")
        } else {
            Self(val)
        }
    }

    pub unsafe fn get_inner_mut(&mut self) -> &mut usize {
        &mut self.0
    }

    pub const fn get_inner(&self) -> usize {
        self.0
    }
}

#[cfg_persistence_derive]
#[derive(Clone)]
#[cfg_educe_debug]
pub enum ProgressMode {
    Batched { size: IPromiseThisIsNonZeroUsize },
}

impl Default for ProgressMode {
    fn default() -> Self {
        Self::Batched {
            size: const { IPromiseThisIsNonZeroUsize::new(500) },
        }
    }
}

#[cfg_persistence_derive]
#[derive(Default, Clone)]
#[cfg_educe_debug]
pub struct AlgoParams {
    pub proposal: GaussianProposal,
    pub accept: RngIter<Percentage>,
    pub progress_mode: ProgressMode,
}

// horrible name but I cant think of something better RN.
pub type AlgoVec = nalgebra::Vector2<f32>;

impl AlgoParams {
    fn propose(&mut self, start_loc: AlgoVec) -> AlgoVec {
        let GaussianProposal {
            sigma,
            rng: ref mut prop_rng,
        } = self.proposal;

        let normal_x = start_loc.x + prop_rng.unwrapped_next() * sigma;
        let normal_y = start_loc.y + prop_rng.unwrapped_next() * sigma;
        AlgoVec::new(normal_x, normal_y)
    }
}

pub type AcceptRecord = RWMHAcceptRecord;

#[expect(clippy::derivable_impls, reason = "AcceptRecord is autogenerated")]
impl Default for AcceptRecord {
    fn default() -> Self {
        Self {
            position: Default::default(),
            remain_count: 0,
            _pad: [0],
        }
    }
}

#[cfg_persistence_derive]
#[derive(Clone)]
#[cfg_educe_debug]
pub struct Rwmh {
    pub current_loc: AcceptRecord,
    pub max_remain_count: u32,
    pub total_point_count: u32,
    // should be HashMap<AlgoVec, i32> or similar,
    // but this is an issue as the f32 in AlgoVec isnt Eq.
    // So IDK how to do this right.
    #[educe(Debug(method(debug_fmt_vec_as_len)))]
    pub history: Vec<AcceptRecord>,
    #[educe(Debug(method(debug_fmt_vec_as_len)))]
    pub rejected_history: Vec<AlgoVec>,
    pub params: AlgoParams,
}

#[cfg(feature = "more_debug_impls")]
fn debug_fmt_vec_as_len<T>(s: &[T], f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Vec<len={len}>", len = s.len())
}

impl Default for Rwmh {
    fn default() -> Self {
        Self {
            // TODO: make start point configurable
            current_loc: AcceptRecord {
                position: [0.0; 2],
                ..Default::default()
            },
            max_remain_count: 0,
            total_point_count: 0,
            // ugly hack around forbidden buffersize zero
            history: vec![RWMHAcceptRecord {
                _pad: [0; 1],
                position: [0.0; 2],
                remain_count: 0,
            }],
            rejected_history: vec![],
            params: Default::default(),
        }
    }
}

impl Rwmh {
    pub fn step(&mut self, target_distr: &GaussianTargetDistr) {
        let current = &mut self.current_loc;
        let proposal = self.params.propose(current.position.into());
        let acceptance_ratio = target_distr.acceptance_ratio(proposal, current.position.into());
        let accept = self.params.accept.unwrapped_next() <= acceptance_ratio;
        // self.current_loc = if accept { proposal } else { current };
        if accept {
            self.total_point_count += self.current_loc.remain_count + 1;
            self.history.push(self.current_loc);
            self.current_loc = AcceptRecord {
                position: [proposal.x, proposal.y],
                remain_count: 0,
                _pad: [0; 1],
            };
        } else {
            current.remain_count += 1;
            self.max_remain_count = self.max_remain_count.max(current.remain_count);
            self.rejected_history.push(proposal);
        };
    }
}
