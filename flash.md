#[contracttype]
pub enum DrugDiscoveryEvent {
    MilestoneReached { research_id: BytesN<32>, milestone: u32 },
    TrialPhaseUpdated { research_id: BytesN<32>, phase: u32 },
    ResultPublished { research_id: BytesN<32>, timestamp: u64 },
}

