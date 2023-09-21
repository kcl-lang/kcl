# KCL Project Governance

As a CNCF sandbox project, the KCL project adheres to the [CNCF Code of Conduct](https://github.com/cncf/foundation/blob/master/code-of-conduct.md).

## Overview

- [KCL Project Governance](#kcl-project-governance)
  - [Overview](#overview)
  - [Maintainer Ship](#maintainer-ship)
  - [Adding Maintainers](#adding-maintainers)
  - [Removal of Inactive Maintainers](#removal-of-inactive-maintainers)
  - [Decision-Making Process](#decision-making-process)
  - [Updating Governance](#updating-governance)

## Maintainer Ship

Maintainers of the KCL project share the responsibility of its success. They have three main responsibilities:

+ Share responsibility for the project's success.
+ Make a long-term investment to improve the project.
+ Spend time on tasks that may not be the most interesting, but are essential for the project's success.

Maintainers often work tirelessly, but their contributions may not always be fully appreciated. While it may be easy to focus on the more exciting and technically advanced features, it is equally important to work on minor bug fixes, small improvements, long-term stability optimizations, and other essential aspects of the project.

## Adding Maintainers

Maintainers are individuals who have shown dedication to the long-term success of the project. Contributors wishing to become maintainers should have actively participated in tackling issues, contributing code, and reviewing proposals and code for a period of at least two months.

Maintainer ship is built on trust, which extends beyond code contributions. It is important for potential maintainers to earn the trust of current maintainers by demonstrating their commitment to the best interests of the project.

Current maintainers hold regular maintainer meetings to identify active contributors who have consistently invested time in the project over the prior months. From this list, if one or more individuals are deemed qualified candidates, a proposal to add them as maintainers can be submitted on GitHub via a pull request. If at least 50% of the maintainers agree with the proposal, the newly added maintainer(s) will be considered valid.

## Removal of Inactive Maintainers

Similar to adding maintainers, existing maintainers can be removed from the active maintainer list. If an existing maintainer meets one of the following conditions, any other maintainer can propose their removal via a pull request:

+ The maintainer has not participated in community activities for more than three months.
+ The maintainer has violated the governance rules more than twice.

Once the above conditions are confirmed, the maintainer can be removed from the list, unless the original maintainer requests to remain and receives at least 50% of the votes from other maintainers.

If a maintainer is removed from the maintaining list, the other maintainers should acknowledge their contribution by adding their name to an alumni section.

## Decision-Making Process

The KCL project is an open-source project that values openness. This means that the KCL repository is the source of truth for every aspect of the project, including values, design, documentation, roadmap, interfaces, etc. If it is part of the project, it should be in the repository.

All decisions, regardless of their size, should follow the following three steps to be considered an update to the project:

1. Open a pull request.
2. Discuss the changes under the pull request.
3. Merge or reject the pull request.

When the KCL project has less than seven maintainers, a pull request (except for adding maintainers) may be merged if it meets the following conditions:

+ At least one maintainer comments "LGTM" (Looks Good To Me) on the pull request.
+ No other maintainers have opposing opinions.

When the KCL project has more than seven maintainers, a pull request (except for adding maintainers) may be merged if it meets the following conditions:

+ At least two maintainers comment "LGTM" (Looks Good To Me) on the pull request.

## Updating Governance

Any substantive updates to the Governance require a supermajority vote from the maintainers.
