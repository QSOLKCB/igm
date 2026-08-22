# Clean-room visualization boundary

## Scope

IGM Phase 2 includes an original Apache-2.0 "fabric / relation" visualization mode.

The implementation is informed by published visualization ideas and research references, including BioFabric, but does **not** copy BioFabric source code, internal implementation, Java classes, UI code, icons, assets, bundled data, or LGPL-covered source into IGM.

## What is reused

Only high-level published ideas and ordinary mathematical/computational concepts are used, such as:

- representing a node/component as a horizontal row;
- representing a relationship as a vertical column spanning participant rows;
- preserving separately visible multiple relationships;
- grouping/filtering relationships by declared class;
- reducing conventional node-link visual clutter.

The IGM renderer was written specifically for the IGM canonical model/profile architecture and its governance invariants.

## What is not reused

IGM does not import, translate, transcribe, port, copy, vendor, or relicense:

- BioFabric Java source;
- BioFabric packages/classes/functions;
- BioFabric layout implementation details beyond published conceptual descriptions;
- BioFabric UI implementation;
- BioFabric artwork/icons;
- BioFabric example data;
- LGPL source code from BioFabric.

## Why this matters

The IGM repository is Apache-2.0 and is intended to remain easy for downstream researchers to fork, modify, replace and embed into their own computational workflows.

A fresh implementation also lets IGM make different design choices required by this project, including:

- one shared canonical state across all views;
- explicit V0/non-clinical labelling;
- relationship-class provenance;
- `INV-VIZ-001` layout/semantics separation;
- `INV-VIZ-002` visual/biological proximity separation;
- future compatibility with typed graphs, hyperedges and replaceable biological profiles.

## References

The research source registry contains the relevant BioFabric method reference and records the access/licensing boundary. Researchers should cite the original BioFabric work when discussing that visualization method.

This document is an engineering provenance record, not legal advice and not a licence interpretation for third-party software.
