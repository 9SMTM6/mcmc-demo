use rand::{Rng, SeedableRng};

use crate::{
    shaders::types::RWMHAcceptRecord, target_distributions::multimodal_gaussian::MultiModalGaussian,
};

use super::{SRngGaussianIter, SRngPercIter};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
pub struct GaussianProposal {
    sigma: f32,
}

impl Default for GaussianProposal {
    fn default() -> Self {
        Self { sigma: 0.2 }
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
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

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
pub enum ProgressMode {
    Batched { size: IPromiseThisIsNonZeroUsize },
}

impl Default for ProgressMode {
    fn default() -> Self {
        Self::Batched {
            size: const { IPromiseThisIsNonZeroUsize::new(2000) },
        }
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default, Clone)]
pub struct AlgoParams {
    pub proposal: GaussianProposal,
    pub progress_mode: ProgressMode,
}

// horrible name but I cant think of something better RN.
pub type AlgoVec = nalgebra::Vector2<f32>;

impl AlgoParams {
    fn propose(
        &self,
        start_loc: AlgoVec,
        gaussian_rng: &mut SRngGaussianIter<impl Rng + SeedableRng>,
    ) -> AlgoVec {
        let GaussianProposal { sigma } = self.proposal;

        let normal_x = start_loc.x + gaussian_rng.unwrapped_next() * sigma;
        let normal_y = start_loc.y + gaussian_rng.unwrapped_next() * sigma;
        AlgoVec::new(normal_x, normal_y)
    }
}

pub type AcceptRecord = RWMHAcceptRecord;

#[allow(clippy::derivable_impls)]
impl Default for AcceptRecord {
    fn default() -> Self {
        Self {
            position: Default::default(),
            remain_count: 0,
            _pad: [0],
        }
    }
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
pub struct Rwmh {
    pub current_loc: AcceptRecord,
    pub max_remain_count: u32,
    pub total_point_count: u32,
    // should be HashMap<AlgoVec, i32> or similar,
    // but this is an issue as the f32 in AlgoVec isnt Eq.
    // So IDK how to do this right.
    pub history: Vec<AcceptRecord>,
    pub rejected_history: Vec<AlgoVec>,
    pub params: AlgoParams,
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
    pub fn step(
        &mut self,
        target_distr: &MultiModalGaussian,
        proposal_rng: &mut SRngGaussianIter<impl Rng + SeedableRng>,
        accept_rng: &mut SRngPercIter<impl Rng + SeedableRng>,
    ) {
        let current = &mut self.current_loc;
        let proposal = self.params.propose(current.position.into(), proposal_rng);
        let acceptance_ratio = target_distr.acceptance_ratio(proposal, current.position.into());
        let accept = accept_rng.unwrapped_next() <= acceptance_ratio;
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
