// GitLab-as-a-DB Adapter
// Uses the repository itself as a persistent data store for reputation

const axios = require('axios');

class GitLabDB {
    constructor(projectId, token) {
        this.projectId = projectId;
        this.token = token;
        this.baseUrl = `https://gitlab.com/api/v4/projects/${projectId}/repository/files`;
    }

    /**
     * Reads reputation from a JSON file in the repository
     */
    async getUserReputation(userId) {
        try {
            const res = await axios.get(`${this.baseUrl}/db%2Freputation.json/raw?ref=master`, {
                headers: { 'PRIVATE-TOKEN': this.token }
            });
            const db = res.data;
            return db[userId] || 0;
        } catch (e) {
            console.log("[DB] No reputation file found, starting at 0.");
            return 0;
        }
    }

    /**
     * Updates user reputation and commits the change back to the repo
     */
    async updateUserReputation(userId, increment) {
        console.log(`[DB] Updating reputation for ${userId} by ${increment}...`);
        
        // 1. Fetch current DB
        let db = {};
        try {
            const res = await axios.get(`${this.baseUrl}/db%2Freputation.json/raw?ref=master`, {
                headers: { 'PRIVATE-TOKEN': this.token }
            });
            db = res.data;
        } catch (e) {}

        // 2. Update logic
        db[userId] = (db[userId] || 0) + increment;

        // 3. Commit back to GitLab
        const payload = {
            branch: 'master',
            commit_message: `update: reputation for ${userId}`,
            content: JSON.stringify(db, null, 2),
            encoding: 'text'
        };

        try {
            await axios.put(`${this.baseUrl}/db%2Freputation.json`, payload, {
                headers: { 'PRIVATE-TOKEN': this.token }
            });
            console.log("[DB] Successfully committed reputation update to repository.");
        } catch (e) {
            // If file doesn't exist, create it
            await axios.post(`${this.baseUrl}/db%2Freputation.json`, payload, {
                headers: { 'PRIVATE-TOKEN': this.token }
            });
            console.log("[DB] Created new reputation file in repository.");
        }
    }

    /**
     * Appends a new social post to the feed in the repository
     */
    async addFeedPost(post) {
        console.log(`[DB] Adding new feed post: ${post.name}...`);
        
        let feed = [];
        try {
            const res = await axios.get(`${this.baseUrl}/db%2Ffeed.json/raw?ref=master`, {
                headers: { 'PRIVATE-TOKEN': this.token }
            });
            feed = res.data;
        } catch (e) {}

        // Add new post to start of feed
        feed.unshift({
            ...post,
            id: Date.now(),
            timestamp: new Date().toISOString()
        });

        // Limit feed size
        if (feed.length > 50) feed = feed.slice(0, 50);

        const payload = {
            branch: 'master',
            commit_message: `social: new update from ${post.name}`,
            content: JSON.stringify(feed, null, 2),
            encoding: 'text'
        };

        try {
            await axios.put(`${this.baseUrl}/db%2Ffeed.json`, payload, {
                headers: { 'PRIVATE-TOKEN': this.token }
            });
        } catch (e) {
            await axios.post(`${this.baseUrl}/db%2Ffeed.json`, payload, {
                headers: { 'PRIVATE-TOKEN': this.token }
            });
        }
    }
}

module.exports = GitLabDB;
