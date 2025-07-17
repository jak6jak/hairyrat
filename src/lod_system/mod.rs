//! Generic LOD System for Bevy
//! 
//! This module provides a flexible and extensible Level of Detail (LOD) system
//! that can be used with various rendering strategies including:
//! - Skeletal animation LOD
//! - Vertex Animation Texture (VAT) LOD
//! - Mesh swapping LOD
//! - Hybrid approaches
//! 
//! # Example
//! ```rust
//! use bevy::prelude::*;
//! use lod_system::*;
//! 
//! #[derive(Component)]
//! struct MyEntity;
//! 
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(LODPlugin::<MyEntity, AnimationLODStrategy>::default())
//!         .run();
//! }
//! ```

pub mod core;
pub mod strategies;
pub mod vat;
pub mod examples;

// Re-export commonly used types
pub use core::{
    LODStrategy, LODLevel, LODState, LODDistance, LODLevels,
    LODProcessingBudget, LODPlugin,
};

pub use strategies::{
    AnimationLODStrategy, AnimationLODConfig, AnimationLODData,
    VATLODStrategy, VATLODConfig, VATLODData,
    MeshSwapLODStrategy, MeshSwapLODConfig, MeshSwapLODData,
    HybridLODStrategy, HybridLODConfig, HybridLODData,
    create_standard_lod_levels, create_aggressive_lod_levels,
};

pub use vat::{
    VATMaterial, VATAnimationState, VATBundle, VATMaterialPlugin,
    create_vat_material, calculate_vat_texture_dimensions,
    VATConfig, SimplifiedVAT,
};

/// Prelude for convenient imports
pub mod prelude {
pub use crate::lod_system::{
    // Core types
    core::{LODStrategy, LODLevel, LODState, LODDistance, LODLevels,
           LODProcessingBudget, LODPlugin},
    
    // Strategies
    strategies::{AnimationLODStrategy, AnimationLODData, VATLODStrategy, 
                 MeshSwapLODStrategy, HybridLODStrategy,
                 create_standard_lod_levels, create_aggressive_lod_levels},
    
    // VAT support
    vat::{VATMaterial, VATAnimationState, VATBundle, VATMaterialPlugin,
          create_vat_material, calculate_vat_texture_dimensions},
};
}
