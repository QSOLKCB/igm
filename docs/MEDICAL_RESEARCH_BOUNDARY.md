# Medical and Research Boundary

## Status

IGM is **exploratory research software**.

It is not represented by this project as a medical device, diagnostic product, clinical decision-support system, treatment-planning system, patient-monitoring system, digital biomarker, or validated predictor of disease or outcome.

This document is a project governance statement, not legal advice. The Apache-2.0 licence remains the controlling software licence. Nothing in this document guarantees that a downstream use is lawful, ethical, clinically appropriate, or outside medical-device regulation.

## Intended purpose of this repository

The intended purpose is to support reproducible computational research into source-informed or hypothetical IgM structural models by providing:

- replaceable model profiles;
- deterministic simulation infrastructure;
- bounded CPU/GPU execution;
- structural and ensemble observables;
- provenance and validation records;
- reproducibility tooling.

The intended user is a researcher, software developer, student, or domain expert evaluating computational models.

The intended output is a **research artifact or computational observation**, not clinical advice.

## Explicitly outside intended purpose

This repository is not intended to:

- diagnose or screen for any disease or condition;
- predict prognosis or patient outcome;
- monitor an individual's disease status;
- recommend treatment, therapy, dosage, timing or clinical management;
- select patients for treatment;
- determine whether a treatment is working;
- estimate a patient's IgM burden or molecular state from personal data;
- replace laboratory testing, imaging, pathology, clinical examination or professional judgement;
- provide emergency or self-care guidance;
- establish a biological mechanism from simulation alone.

## Why the boundary is explicit

Under Australian medical-device regulation, **intended purpose matters**. A downstream version of the software may cross into regulated software if its intended purpose changes toward diagnosis, monitoring, prediction, prognosis, treatment, investigation of anatomy/physiology for a medical purpose, or related clinical functions.

Therefore, removing this disclaimer while adding patient-specific or clinical functionality is not a cosmetic documentation change. It is a change in product/research intent that requires dedicated legal, regulatory, ethics, quality and scientific review.

## Computational validity is not biological validity

A model can be:

- mathematically well-defined;
- numerically stable;
- deterministic;
- CPU/GPU reproducible;
- internally constraint-consistent;

and still be biologically wrong.

The repository therefore separates:

```text
computational correctness != biological validation != clinical validity
```

No automated test may promote one category into another.

## Personal experience is not a validation dataset

Personal observations, anecdotes, symptom histories, treatment histories, visual resemblance, intuition and lived experience can motivate research questions. They are not treated by this repository as a substitute for appropriately governed research data or independent evidence.

## Public repository data rule

This repository is not an approved store for patient or participant data.

Do not commit:

- names or contact details;
- medical record numbers or identifiers;
- pathology results linked to a person;
- treatment histories linked to a person;
- genomic or sequence data linked to a participant;
- clinical images;
- coded participant datasets;
- dates or combinations of attributes that create re-identification risk;
- private hospital, clinic or research data.

Even data described as "de-identified" may remain re-identifiable. Human research data must be handled under the applicable protocol, data-management plan, privacy requirements and institutional governance.

## Structural biology sources

Public structural sources may be used when their licence/access conditions permit. Each biologically meaningful parameter should identify what its source supports and distinguish:

- observed;
- source-derived;
- calibrated;
- assumed;
- inferred;
- unknown.

The simulator should prefer an explicit unknown over an invented plausible value.

## Downstream researcher responsibilities

Anyone adapting IGM for a real research program is responsible for determining, with their institution and qualified advisers where appropriate:

- whether human research ethics review is required;
- whether site governance approval is required;
- whether consent/waiver requirements apply;
- whether privacy legislation or data-sharing agreements apply;
- whether the intended use is regulated as a medical device or therapeutic good;
- whether clinical-trial or medical-device investigation requirements apply;
- what quality, cybersecurity, records, safety and validation standards are applicable;
- whether publication and dissemination claims accurately reflect evidence.

## No institutional endorsement

References to Flinders University, SA Health, NHMRC, TGA, OAIC or other institutions describe governance context only. They do not imply collaboration, approval, sponsorship, validation or endorsement.

## Researcher promotion rule

The open project can supply tools and reproducible computation. Biological and clinical promotion requires evidence external to the runtime.

A downstream researcher may replace a schematic profile with stronger cryo-EM, MD, biochemical or calibrated inputs and conduct appropriate validation. That work should be separately identified with its investigators, protocol, data sources, approvals and claims.
