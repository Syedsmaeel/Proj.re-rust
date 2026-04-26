// Git Integration Service
// Aggregates data from GitHub, GitLab, and Bitbucket APIs
const axios = require('axios');

class GitIntegrator {
    constructor() {
        this.services = ['github', 'gitlab'];
    }

    /**
     * Fetches a "Global Pulse" of the latest public activity from GitHub and GitLab
     */
    async getGlobalPulse() {
        console.log("[GitIntegrator] Fetching Global Pulse from the mesh...");
        const pulse = [];

        try {
            // 1. GitHub Public Events
            const ghRes = await axios.get('https://api.github.com/events?per_page=5', {
                headers: { 'Accept': 'application/vnd.github.v3+json' }
            });
            ghRes.data.forEach(event => {
                pulse.push({
                    name: event.repo.name,
                    author: event.actor.login,
                    service: 'github',
                    description: `Detected ${event.type} in global scope.`,
                    timestamp: new Date().toISOString()
                });
            });

            // 2. GitLab Public Projects
            const glRes = await axios.get('https://gitlab.com/api/v4/projects?visibility=public&sort=desc&per_page=5');
            glRes.data.forEach(project => {
                pulse.push({
                    name: project.name_with_namespace,
                    author: project.namespace.name,
                    service: 'gitlab',
                    description: `New public project detected: ${project.description || 'No description'}`,
                    timestamp: project.created_at
                });
            });
        } catch (e) {
            console.error("[GitIntegrator] Pulse fetch failed:", e.message);
        }

        return pulse;
    }

    /**
     * Fetches repositories and recent activity from all configured services.
     * @param {Object} tokens - API tokens for services
     */
    async getUnifiedFeed(tokens) {
        console.log("[GitIntegrator] Aggregating activity from", this.services);
        
        // Mock implementation of aggregate fetching
        const results = this.services.map(service => ({
            service,
            repos: [
                { name: `${service}-repo-1`, updated: new Date().toISOString() },
                { name: `${service}-repo-2`, updated: new Date().toISOString() }
            ]
        }));

        return results;
    }
}

module.exports = new GitIntegrator();
