# Unified Dev-Social Hub - Wiki

## 1. Introduction
Unified Dev-Social Hub is a decentralized, AI-driven, and Bitcoin-incentivized developer ecosystem. It is designed to be autonomous, privacy-focused, and 100% free to host.

## 2. Platform Architecture
- **Frontend:** Hosted on **GitLab Pages**, serving a modern, static dashboard.
- **In-Browser Backend:** Utilizes Service Workers to handle API requests and dynamic data without needing a central server.
- **Agentic Cloud Worker:** A background **GitLab CI/CD Job** that acts as the platform's "Brain," performing AI audits, managing database commits, and executing payouts.

## 3. How to Interact with the AI (Direct-to-Brain)
You can trigger the AI Agent directly from your terminal, bypassing the dashboard entirely:

```bash
curl -X POST \
     -F "token=<YOUR_TRIGGER_TOKEN>" \
     -F "ref=master" \
     -F "variables[PROMPT]=Your Question Here" \
     https://gitlab.com/api/v4/projects/81630798/trigger/pipeline
```

- **Token:** Your GitLab Pipeline Trigger Token.
- **Prompt:** The task or question you want the AI to process.
- **Result:** The agent will process the request in the cloud, audit your repo, and commit the response to your `db/chat.json` file.

## 4. Satoshi Reward Mechanism
The platform includes an automated payment rail for high-quality contributions:
- The AI Agent performs an audit.
- If the audit is `PASSED`, the agent triggers a payout via the `LightningRail` service.
- **Payout:** 10,000,000 Satoshis per contribution.

## 5. Security & Privacy
- **E2EE:** All interactions are encrypted.
- **No Third-Party:** No external server or database is used—only your repository and your browser.
- **Sovereignty:** You own your AI, your data, and your reputation engine.
