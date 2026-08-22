# Australian Ethics and Regulatory Foundation

## Purpose

This document gives IGM contributors and downstream researchers a conservative Australian governance baseline for deciding when exploratory simulation work has crossed into human research, health-information handling, clinical investigation, or regulated medical-device activity.

It is **not legal advice**, does not replace an institution's research office/HREC, and does not certify compliance. Australian requirements change over time; researchers must verify the current official sources before commencing a study.

## 1. National Statement on Ethical Conduct in Human Research 2025

Official source:

- https://www.nhmrc.gov.au/research-policy/ethics/national-statement-ethical-conduct-human-research
- https://www.nhmrc.gov.au/about-us/publications/national-statement-ethical-conduct-human-research-2025

The 2025 National Statement is the current Australian national ethics framework for human research and took effect on 23 June 2026.

IGM's default public-development workflow is deliberately designed to avoid human-participant research by using synthetic fixtures and public structural sources. If a downstream study introduces human participants or participant data, the research team must determine the required ethics-review pathway with the relevant institution before starting that research activity.

### Repository implications

- no assumption that open-source availability equals ethics approval;
- no participant recruitment through repository tooling;
- no participant data in public fixtures;
- no patient-specific modelling by default;
- no HREC approval claims unless a real approval exists and its scope is accurately represented;
- ethics approval, where required, belongs to the actual research protocol, investigators and institutions, not to the generic codebase.

## 2. Australian Code for the Responsible Conduct of Research 2018

Official source:

- https://www.nhmrc.gov.au/about-us/publications/australian-code-responsible-conduct-research-2018

The Code establishes a principles-based framework for responsible Australian research and is supported by guidance on matters including research data, authorship, collaboration, conflicts of interest, peer review, publication and dissemination.

### Repository implications

IGM should support:

- honest representation of what evidence establishes;
- transparent assumptions and uncertainty;
- reproducible methods;
- traceable source and contributor attribution;
- preservation of relevant validation/rejection evidence;
- appropriate research-data management;
- disclosure of conflicts and interests in downstream research where applicable;
- correction of errors rather than preservation of convenient claims.

## 3. Privacy Act 1988 and Australian Privacy Principles

Official source:

- https://www.oaic.gov.au/privacy/australian-privacy-principles/australian-privacy-principles-guidelines

The Australian Privacy Principles address matters including collection, notification, use/disclosure, data quality, security, access and correction of personal information.

### Repository rule

The public repository is **not an approved health-information repository**.

Do not commit human research or clinical datasets merely because fields have been removed. Re-identification risk depends on the data and context, not only on whether a name is present.

A downstream research program that handles health/personal information should establish appropriate institutional storage, access control, retention/destruction, consent/waiver, data-sharing, breach-response and cross-border arrangements before data enters the workflow.

## 4. TGA software-based medical-device boundary

Official sources:

- https://www.tga.gov.au/resources/guidance/understanding-how-we-regulate-software-based-medical-devices
- https://www.tga.gov.au/products/medical-devices/software-and-artificial-intelligence-ai/overview
- https://www.tga.gov.au/products/medical-devices/software-and-artificial-intelligence-ai/overview/software-based-medical-device-exclusions

TGA guidance makes intended purpose central to determining whether software meets the definition of a medical device. Software intended for functions such as diagnosis, prevention, monitoring, prediction, prognosis, treatment, or medical investigation of anatomy/physiology may fall within medical-device regulation unless an exclusion/exemption applies.

### Current IGM intended purpose

The upstream IGM repository is framed as exploratory structural research infrastructure and explicitly excludes patient-specific diagnostic, prognostic, monitoring, treatment and clinical decision-support purposes.

This statement is a project design boundary, **not a binding TGA classification decision**.

### Change-control trigger

A proposal to add any of the following must trigger dedicated regulatory/ethics review before implementation or release:

- patient-specific input intended to produce a medical conclusion;
- diagnostic/screening classification;
- prognosis or outcome prediction;
- treatment recommendation or selection;
- disease monitoring;
- clinical decision support;
- reporting intended to influence patient management;
- software supplied with a medical intended purpose.

## 5. Standards for future regulated development

TGA notes that international standards can support safety, quality and cybersecurity for medical-device software, while also noting that the standards it lists are best-practice references rather than automatically mandatory or TGA-endorsed requirements.

Official source:

- https://www.tga.gov.au/products/medical-devices/software-and-artificial-intelligence-ai/overview/standards-software-based-medical-devices

Examples potentially relevant to a **future regulated downstream project**, depending on intended purpose and classification, include:

- IEC 62304 — medical device software lifecycle processes;
- ISO 14971 — medical-device risk management;
- ISO 13485 — medical-device quality management systems;
- IEC 81001-5-1 / related health-software cybersecurity practices where applicable;
- ISO 14155 — clinical investigation of medical devices involving human subjects where applicable.

IGM does not claim conformance with these standards in Phase 1. They are intentionally separated from exploratory research so later regulated work can adopt the correct quality framework rather than retroactively pretending research code was a certified medical product.

## 6. Flinders University / SA Health context

Official Flinders human-ethics source:

- https://staff.flinders.edu.au/research/integrity/human-ethics

Flinders states that research involving human participants and/or their data requires the relevant human-research ethics process before the study commences. Flinders also directs research involving SA Health sites, or participants/data accessed through SA Health sites, to an SA Health HREC pathway.

South Australian research-governance portal:

- https://gems.sahealth.sa.gov.au/

### Repository implication

A future collaboration with Flinders researchers or a study using SA Health sites/data would be a **new governed research activity**. The generic open-source repository cannot pre-authorise that activity.

## 7. Clinical investigations

The 2025 National Statement references current Good Clinical Practice requirements and, where required, ISO 14155 and TGA requirements for clinical investigation of medical devices.

IGM Phase 1 does not conduct a clinical investigation.

Any downstream clinical study must establish its own protocol, sponsorship/investigator responsibilities, ethics approval, governance, registration/notification requirements, safety reporting, monitoring, data management and statistical validation as applicable.

## 8. Consumer/community involvement

Health and medical research can benefit from consumer and community involvement, but lived experience must be incorporated ethically and without being mistaken for independent validation.

Potential future community contribution should be handled through a defined research/community-engagement process rather than by embedding personal clinical histories in source code or public issues.

## 9. Conservative decision table

| Proposed activity | Upstream IGM default | Governance response |
|---|---|---|
| synthetic structural fixture | allowed | document assumptions |
| public structural database input | allowed subject to source/licence | record provenance |
| literature-derived parameter | allowed | record exact support/uncertainty |
| private laboratory data | not default | data agreement + governance review |
| human participant data | prohibited in public repo | ethics/governance process first |
| patient-specific prediction | outside intended purpose | regulatory/clinical review required |
| treatment recommendation | outside intended purpose | do not implement as ordinary research feature |
| clinical trial/investigation | outside Phase 1 | institutional + regulatory framework required |
| GPU performance study | allowed | no biological/clinical promotion |

## 10. Review rule

Before a major release, re-check official NHMRC, OAIC and TGA guidance. Do not freeze outdated regulatory summaries into code as if they were permanent law.
