const express = require('express');
const router = express.Router();
const { processGitActivity } = require('../api/connector');

// Simulated persistent storage
const db = {
    users: {}, // { username: { reputation: 0, lastReward: null } }
};

// GET Dashboard: Aggregates Git activity, user auth, and Docker status
router.get('/dashboard', async (req, res) => {
    const userId = req.headers['x-user-id'] || 'guest';
    
    // Aggregation Logic
    const gitActivity = await processGitActivity(userId);
    
    if (!db.users[userId]) {
        db.users[userId] = { reputation: 0 };
    }

    res.json({
        user: { id: userId, reputation: db.users[userId].reputation },
        gitActivity,
        status: "success"
    });
});

module.exports = router;
