// API Connector: Git -> Docker++ Feed (Refined)
// Now includes a simulated AI validation step via OpenClaw before triggering rewards

const gitIntegrator = require('../../../../packages/git-integrator');
const { triggerReward } = require('../services/rewards');
const openClaw = require('../../../../packages/openclaw');

async function processGitActivity(userId) {
    console.log(`[Connector] Analyzing and syncing Git activity for user: ${userId}`);
    
    // 1. Fetch data from Git Integrator
    const gitData = await gitIntegrator.getUnifiedFeed();
    
    // 2. Validate/Audit activity using OpenClaw (The Intelligence Engine)
    // We'll simulate checking a repo description as a Docker configuration
    const enhancedPosts = [];
    for (const service of gitData) {
        for (const repo of service.repos) {
            const auditResult = await openClaw.auditDockerfile(repo.name); // Using repo name as proxy for config content
            
            enhancedPosts.push({
                ...repo,
                service: service.service,
                audit: auditResult
            });

            // 3. Conditional Reward Trigger: Only reward if audit status is PASSED
            if (auditResult.status === 'PASSED') {
                await triggerReward("bc1qs4rr5prccv63fsws2yx6c8pctx50uzkvc8yyhj", `High-quality repo interaction: ${repo.name}`);
            }
        }
    }

    return enhancedPosts;
}

module.exports = { processGitActivity };
