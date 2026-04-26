require('dotenv').config();
const express = require('express');
const app = express();
const path = require('path');
const PORT = process.env.PORT || 3000;
const postRoutes = require('./routes/posts');
const authRoutes = require('./routes/auth');
const dashboardRoutes = require('./routes/dashboard');

app.use(express.json());
app.use(express.static(path.join(__dirname, '../public')));

app.use('/api/posts', postRoutes);
app.use('/api/auth', authRoutes);
app.use('/api/dashboard', dashboardRoutes);

app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, '../public/index.html'));
});

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
