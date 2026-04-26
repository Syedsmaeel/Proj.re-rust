const express = require('express');
const router = express.Router();

// Mock database
let posts = [];

// Create a new container post
router.post('/', (req, res) => {
    const { userId, dockerfileContent, description } = req.body;
    const newPost = {
        id: Date.now().toString(),
        userId,
        dockerfileContent,
        description,
        createdAt: new Date()
    };
    posts.push(newPost);
    res.status(201).json(newPost);
});

// Get all container posts
router.get('/', (req, res) => {
    res.json(posts);
});

module.exports = router;
