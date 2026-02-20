//! Integration test: asset storage and deduplication.
//!
//! Verifies that content-addressed assets are stored, retrieved, and
//! deduplicated correctly, including integration with snapshot workflows.

mod fixtures;

use dit_core::assets;
use dit_core::repository::{CommitOptions, DitRepository};
use dit_core::types::*;
use tempfile::TempDir;

// ── Basic deduplication ─────────────────────────────────────────────────────

#[test]
fn identical_content_produces_same_hash() {
    let data = b"identical PNG bytes";
    let hash1 = assets::compute_hash(data);
    let hash2 = assets::compute_hash(data);
    assert_eq!(hash1, hash2);
}

#[test]
fn different_content_produces_different_hash() {
    let hash1 = assets::compute_hash(b"image A");
    let hash2 = assets::compute_hash(b"image B");
    assert_ne!(hash1, hash2);
}

#[test]
fn store_same_content_twice_returns_same_ref() {
    let tmp = TempDir::new().unwrap();
    let data = b"duplicate asset content";

    let ref1 = assets::store_asset(tmp.path(), data).unwrap();
    let ref2 = assets::store_asset(tmp.path(), data).unwrap();
    assert_eq!(ref1, ref2, "same content should produce same asset reference");
}

#[test]
fn dedup_does_not_create_extra_files() {
    let tmp = TempDir::new().unwrap();
    let data = b"deduplicate this";

    assets::store_asset(tmp.path(), data).unwrap();
    assets::store_asset(tmp.path(), data).unwrap();
    assets::store_asset(tmp.path(), data).unwrap();

    // Only one file should exist in assets dir
    let assets_dir = tmp.path().join(DitPaths::ASSETS_DIR);
    let count = std::fs::read_dir(&assets_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .count();
    assert_eq!(count, 1, "deduplication should result in exactly one file");
}

// ── Store and retrieve ──────────────────────────────────────────────────────

#[test]
fn store_and_retrieve_various_sizes() {
    let tmp = TempDir::new().unwrap();

    // Small (1 byte)
    let small = vec![0x42u8];
    let ref_small = assets::store_asset(tmp.path(), &small).unwrap();
    let retrieved_small = assets::retrieve_asset(tmp.path(), &ref_small).unwrap();
    assert_eq!(retrieved_small, small);

    // Medium (1 KB)
    let medium: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let ref_medium = assets::store_asset(tmp.path(), &medium).unwrap();
    let retrieved_medium = assets::retrieve_asset(tmp.path(), &ref_medium).unwrap();
    assert_eq!(retrieved_medium, medium);

    // Large (1 MB)
    let large: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
    let ref_large = assets::store_asset(tmp.path(), &large).unwrap();
    let retrieved_large = assets::retrieve_asset(tmp.path(), &ref_large).unwrap();
    assert_eq!(retrieved_large, large);
}

#[test]
fn asset_ref_format() {
    let data = b"test";
    let ref_str = assets::create_asset_ref(data);
    assert!(ref_str.starts_with("sha256:"), "ref should start with sha256: prefix");
    // SHA-256 hex is 64 characters
    let hash_part = &ref_str["sha256:".len()..];
    assert_eq!(hash_part.len(), 64, "SHA-256 hex should be 64 chars");
    assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()), "hash should be hex");
}

#[test]
fn asset_exists_check() {
    let tmp = TempDir::new().unwrap();
    let data = b"check existence";

    let ref_str = assets::store_asset(tmp.path(), data).unwrap();
    assert!(assets::asset_exists(tmp.path(), &ref_str));
    assert!(!assets::asset_exists(tmp.path(), "sha256:0000000000000000000000000000000000000000000000000000000000000000"));
    assert!(!assets::asset_exists(tmp.path(), "invalid-ref-format"));
}

// ── Multiple different assets ───────────────────────────────────────────────

#[test]
fn multiple_different_assets_all_stored() {
    let tmp = TempDir::new().unwrap();

    let assets_data: Vec<Vec<u8>> = (0..5)
        .map(|i| format!("asset content {i}").into_bytes())
        .collect();

    let refs: Vec<String> = assets_data
        .iter()
        .map(|d| assets::store_asset(tmp.path(), d).unwrap())
        .collect();

    // All refs should be unique
    for (i, r1) in refs.iter().enumerate() {
        for (j, r2) in refs.iter().enumerate() {
            if i != j {
                assert_ne!(r1, r2, "different content should have different refs");
            }
        }
    }

    // All should be retrievable
    for (data, ref_str) in assets_data.iter().zip(refs.iter()) {
        let retrieved = assets::retrieve_asset(tmp.path(), ref_str).unwrap();
        assert_eq!(&retrieved, data);
    }
}

// ── Assets with repository workflow ─────────────────────────────────────────

#[test]
fn assets_survive_commit_workflow() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(
        tmp.path(),
        DitConfig {
            file_key: "asset-test".into(),
            name: "Asset Test".into(),
            figma_token: None,
            schema_version: 1,
        },
    )
    .unwrap();

    // Store an asset
    let image_data = b"PNG image bytes for testing";
    let asset_ref = assets::store_asset(repo.root(), image_data).unwrap();

    // Create snapshot that references the asset
    let snap = DitSnapshot {
        project: fixtures::project(),
        pages: vec![DitPage {
            id: "0:1".into(),
            name: "Page with image".into(),
            background_color: None,
            children: vec![fixtures::image_node(&asset_ref)],
        }],
        components: None,
        component_sets: None,
        styles: None,
    };

    // Commit
    repo.commit(&snap, "add image", &CommitOptions::default()).unwrap();

    // Verify asset is still accessible
    assert!(assets::asset_exists(repo.root(), &asset_ref));
    let retrieved = assets::retrieve_asset(repo.root(), &asset_ref).unwrap();
    assert_eq!(retrieved, image_data);

    // Verify snapshot references the asset
    let loaded = repo.read_current_snapshot().unwrap();
    let image_fill = &loaded.pages[0].children[0].fills.as_ref().unwrap()[0];
    assert_eq!(image_fill.image_ref.as_deref(), Some(asset_ref.as_str()));
}

#[test]
fn shared_assets_across_nodes() {
    let tmp = TempDir::new().unwrap();

    // Store one asset
    let shared_data = b"shared image content";
    let ref_str = assets::store_asset(tmp.path(), shared_data).unwrap();

    // Two nodes referencing the same asset
    let node1 = fixtures::image_node(&ref_str);
    let node2 = fixtures::image_node(&ref_str);

    // Both should reference the same asset
    let ref1 = node1.fills.as_ref().unwrap()[0].image_ref.as_ref().unwrap();
    let ref2 = node2.fills.as_ref().unwrap()[0].image_ref.as_ref().unwrap();
    assert_eq!(ref1, ref2);

    // Only one file on disk
    let assets_dir = tmp.path().join(DitPaths::ASSETS_DIR);
    let count = std::fs::read_dir(&assets_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .count();
    assert_eq!(count, 1);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn empty_content_is_valid_asset() {
    let tmp = TempDir::new().unwrap();
    let data = b"";
    let ref_str = assets::store_asset(tmp.path(), data).unwrap();
    let retrieved = assets::retrieve_asset(tmp.path(), &ref_str).unwrap();
    assert_eq!(retrieved, data);
}

#[test]
fn binary_content_round_trips() {
    let tmp = TempDir::new().unwrap();
    // All byte values
    let data: Vec<u8> = (0..=255).collect();
    let ref_str = assets::store_asset(tmp.path(), &data).unwrap();
    let retrieved = assets::retrieve_asset(tmp.path(), &ref_str).unwrap();
    assert_eq!(retrieved, data);
}

#[test]
fn retrieve_nonexistent_asset_errors() {
    let tmp = TempDir::new().unwrap();
    let result = assets::retrieve_asset(
        tmp.path(),
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    assert!(result.is_err());
}
