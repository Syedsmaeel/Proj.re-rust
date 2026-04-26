// Claude Connector for GitLab
// This module provides an interface between Claude's agentic loop
// and your GitLab project via the GitLab API.

const axios = require('axios');

class GitLabConnector {
    constructor(projectId, token) {
        this.projectId = projectId;
        this.client = axios.create({
            baseURL: `https://gitlab.com/api/v4/projects/${projectId}`,
            headers: { 'PRIVATE-TOKEN': token }
        });
    }

    async getFile(path, branch = 'master') {
        const encodedPath = encodeURIComponent(path);
        const res = await this.client.get(`/repository/files/${encodedPath}/raw?ref=${branch}`);
        return res.data;
    }

    async commitChanges(message, actions) {
        const res = await this.client.post('/repository/commits', {
            branch: 'master',
            commit_message: message,
            actions: actions
        });
        return res.data;
    }
}

module.exports = GitLabConnector;
