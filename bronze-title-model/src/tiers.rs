#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleTier {
    Extractive,
    Smol135,
    Smol360,
    Qwen05,
}

pub struct TierSpec {
    pub tier: TitleTier,
    pub filename: &'static str,
    pub sha256_hex: &'static str,
    pub hf_base_repo: &'static str,
    pub hf_gguf_repo: &'static str,
    pub hf_revision: &'static str,
    pub bytes: u64,
}

pub const GGUF_TIERS: &[TierSpec] = &[
    TierSpec {
        tier: TitleTier::Smol135,
        filename: "SmolLM2-135M-Instruct-Q4_K_M.gguf",
        sha256_hex: "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d",
        hf_base_repo: "HuggingFaceTB/SmolLM2-135M-Instruct",
        hf_gguf_repo: "bartowski/SmolLM2-135M-Instruct-GGUF",
        hf_revision: "09816acd5d99df7be770d85ea30822623dab342c",
        bytes: 105_454_432,
    },
    TierSpec {
        tier: TitleTier::Smol360,
        filename: "SmolLM2-360M-Instruct-Q4_K_M.gguf",
        sha256_hex: "2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2",
        hf_base_repo: "HuggingFaceTB/SmolLM2-360M-Instruct",
        hf_gguf_repo: "bartowski/SmolLM2-360M-Instruct-GGUF",
        hf_revision: "ab928a97ee49f3a015f35194879f68211291d6ca",
        bytes: 270_590_880,
    },
    TierSpec {
        tier: TitleTier::Qwen05,
        filename: "qwen2.5-0.5b-instruct-q4_k_m.gguf",
        sha256_hex: "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db",
        hf_base_repo: "Qwen/Qwen2.5-0.5B-Instruct",
        hf_gguf_repo: "Qwen/Qwen2.5-0.5B-Instruct-GGUF",
        hf_revision: "9217f5db79a29953eb74d5343926648285ec7e67",
        bytes: 491_400_032,
    },
];

const GIB: u64 = 1024 * 1024 * 1024;

impl TitleTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Extractive => "extractive",
            Self::Smol135 => "smol-135",
            Self::Smol360 => "smol-360",
            Self::Qwen05 => "qwen-05",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "extractive" => Some(Self::Extractive),
            "smol-135" => Some(Self::Smol135),
            "smol-360" => Some(Self::Smol360),
            "qwen-05" => Some(Self::Qwen05),
            _ => None,
        }
    }

    pub const fn rank(self) -> u8 {
        match self {
            Self::Extractive => 0,
            Self::Smol135 => 1,
            Self::Smol360 => 2,
            Self::Qwen05 => 3,
        }
    }

    pub fn spec(self) -> Option<&'static TierSpec> {
        GGUF_TIERS.iter().find(|spec| spec.tier == self)
    }

    pub fn vendor_command(self) -> &'static str {
        match self {
            Self::Extractive => "",
            Self::Smol135 => "sh bronze-title-model/scripts/vendor-gguf.sh smol-135",
            Self::Smol360 => "sh bronze-title-model/scripts/vendor-gguf.sh smol-360",
            Self::Qwen05 => "sh bronze-title-model/scripts/vendor-gguf.sh qwen-05",
        }
    }
}

pub fn spec_for_filename(filename: &str) -> Option<&'static TierSpec> {
    GGUF_TIERS.iter().find(|spec| spec.filename == filename)
}

pub fn auto_pick_title_tier(ram_bytes: Option<u64>, present_gguf: &[TitleTier]) -> TitleTier {
    let Some(ram) = ram_bytes else {
        return TitleTier::Extractive;
    };
    if ram < 8 * GIB || present_gguf.is_empty() {
        return TitleTier::Extractive;
    }
    let ceiling = if ram < 16 * GIB {
        TitleTier::Smol135
    } else if ram < 32 * GIB {
        TitleTier::Smol360
    } else {
        TitleTier::Qwen05
    };
    for candidate in [TitleTier::Qwen05, TitleTier::Smol360, TitleTier::Smol135] {
        if candidate.rank() <= ceiling.rank() && present_gguf.contains(&candidate) {
            return candidate;
        }
    }
    TitleTier::Extractive
}

#[cfg(test)]
mod tier_tests {
    use super::*;

    #[test]
    fn auto_pick_uses_ram_band_and_present_files_only() {
        let all = [TitleTier::Smol135, TitleTier::Smol360, TitleTier::Qwen05];
        assert_eq!(
            auto_pick_title_tier(Some(7 * GIB), &all),
            TitleTier::Extractive
        );
        assert_eq!(
            auto_pick_title_tier(Some(8 * GIB), &all),
            TitleTier::Smol135
        );
        assert_eq!(
            auto_pick_title_tier(Some(16 * GIB), &all),
            TitleTier::Smol360
        );
        assert_eq!(
            auto_pick_title_tier(Some(32 * GIB), &all),
            TitleTier::Qwen05
        );
        assert_eq!(
            auto_pick_title_tier(Some(32 * GIB), &[TitleTier::Smol360]),
            TitleTier::Smol360
        );
        assert_eq!(
            auto_pick_title_tier(Some(16 * GIB), &[TitleTier::Qwen05]),
            TitleTier::Extractive
        );
        assert_eq!(
            auto_pick_title_tier(Some(10 * GIB), &[TitleTier::Smol360]),
            TitleTier::Extractive
        );
        assert_eq!(auto_pick_title_tier(None, &all), TitleTier::Extractive);
        assert_eq!(
            auto_pick_title_tier(Some(64 * GIB), &[]),
            TitleTier::Extractive
        );
    }

    #[test]
    fn qwen_pin_matches_official_q4_k_m() {
        let qwen = TitleTier::Qwen05.spec().expect("qwen");
        assert_eq!(qwen.filename, "qwen2.5-0.5b-instruct-q4_k_m.gguf");
        assert_eq!(
            qwen.sha256_hex,
            "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db"
        );
        assert_eq!(qwen.hf_gguf_repo, "Qwen/Qwen2.5-0.5B-Instruct-GGUF");
        assert_eq!(qwen.hf_revision, "9217f5db79a29953eb74d5343926648285ec7e67");
        assert_eq!(qwen.bytes, 491_400_032);
        assert!(!qwen.filename.contains("Qwen3"));
    }
}
