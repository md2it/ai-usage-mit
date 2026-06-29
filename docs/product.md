# PRODUCT

## Problem

AI spending is hard to control when usage is spread across multiple CLIs, models, and providers. API billing dashboards only help when requests go through API accounts, while subscription plans usually show quotas indirectly, inconsistently, or only inside vendor interfaces.

This creates several practical risks:

- Usage only becomes noticeable after hitting the limit
- Different providers use different quota rules and reset windows
- Token and request consumption is hard to compare across tools
- Paid overages or forced upgrades can happen before the user sees a trend
- No working free solution was found:
   - Most tools show API spending, not subscription plan usage
   - Too heavy
   - Too expensive
   - Require routing traffic through another vendor
   - Many simply do not work
   - Many are difficult to run and configure

## Target solution

A lightweight local tracker focused on AI usage through CLIs.

## User capabilities

From the user's point of view, the system provides five core capabilities:

1. Get limits

   The user can see the current usage limits that apply to their AI tools, accounts, plans, or providers.

2. Get usage

   The user can see how much of the available limit has already been used for a selected tool, provider, model, account, or time window.

3. Check access

   The user can verify whether the system has enough access to read the required usage and limit information from the relevant source.

4. Configure and receive notifications

   The user can configure notifications and then receive alerts when usage reaches important thresholds or when relevant limit conditions change.

5. Hard usage blocking

   The user can enable hard blocking so that usage is stopped when a defined limit or policy condition is reached.

Notifications and hard blocking are part of the target product scope, but their detailed setup and behavior will be defined later.

## Business process

The product flow can be described as a business-readable technical process:

1. Get data from information sources

   The system works with a defined set of information sources. Each source can have its own request format, access method, data location, limitations, fallback paths, and reliability constraints. The catalog of these source-specific approaches is maintained in [docs/get-info](get-info/).

2. Convert raw data into structured information

   The system processes the raw information received from each source and extracts normalized, structured data from it. This structured data should represent business-relevant facts such as available limits, used volume, reset periods, account context, provider context, and access status.

3. Provide user-facing results

   The system exposes the structured information to the user as clear answers about limits and usage. Notification configuration and hard blocking will use the same structured information later, but they are outside the current process detail.
