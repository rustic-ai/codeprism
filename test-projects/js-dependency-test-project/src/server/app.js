/**
 * Express.js application for testing Node.js patterns
 * This demonstrates various Node.js backend patterns, middleware, routing, and database integration
 */

const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const compression = require('compression');
const rateLimit = require('express-rate-limit');
const morgan = require('morgan');
const swaggerUi = require('swagger-ui-express');
const mongoose = require('mongoose');
const redis = require('redis');
const jwt = require('jsonwebtoken');
const bcrypt = require('bcrypt');
const { body, validationResult } = require('express-validator');
const winston = require('winston');

// Import custom modules
const UserService = require('./services/UserService');
const AuthService = require('./services/AuthService');
const DatabaseService = require('./services/DatabaseService');
const CacheService = require('./services/CacheService');
const emailService = require('./services/EmailService');
const { logger } = require('./utils/logger');
const config = require('./config/config');

// Error handling middleware
const errorHandler = require('./middleware/errorHandler');
const authMiddleware = require('./middleware/auth');
const validationMiddleware = require('./middleware/validation');

// Models
const User = require('./models/User');
const Product = require('./models/Product');
const Order = require('./models/Order');

// Create Express application
const app = express();

// Environment configuration
const NODE_ENV = process.env.NODE_ENV || 'development';
const PORT = process.env.PORT || 3000;
const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key';
const MONGODB_URI = process.env.MONGODB_URI || 'mongodb://localhost:27017/testdb';
const REDIS_URL = process.env.REDIS_URL || 'redis://localhost:6379';

// Database connections (multiple database pattern)
const mongooseConnection = mongoose.createConnection(MONGODB_URI);
const redisClient = redis.createClient({ url: REDIS_URL });

// Security middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
    },
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true
  }
}));

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP, please try again later.',
  standardHeaders: true,
  legacyHeaders: false,
});

app.use(limiter);

// CORS configuration
const corsOptions = {
  origin: process.env.ALLOWED_ORIGINS?.split(',') || ['http://localhost:3000'],
  credentials: true,
  optionsSuccessStatus: 200,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With']
};

app.use(cors(corsOptions));

// Body parsing middleware
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));
app.use(compression());

// Logging middleware
app.use(morgan('combined', {
  stream: {
    write: (message) => logger.info(message.trim())
  }
}));

// Session and cookie middleware
const session = require('express-session');
const MongoStore = require('connect-mongo');

app.use(session({
  secret: process.env.SESSION_SECRET || 'session-secret',
  resave: false,
  saveUninitialized: false,
  store: MongoStore.create({
    mongoUrl: MONGODB_URI,
    ttl: 24 * 60 * 60 // 1 day
  }),
  cookie: {
    secure: NODE_ENV === 'production',
    httpOnly: true,
    maxAge: 24 * 60 * 60 * 1000 // 1 day
  }
}));

// Initialize services (Dependency Injection pattern)
const services = {
  userService: new UserService(mongooseConnection),
  authService: new AuthService(),
  databaseService: new DatabaseService(mongooseConnection),
  cacheService: new CacheService(redisClient)
};

// Make services available to routes
app.locals.services = services;

// Authentication middleware
const authenticateToken = (req, res, next) => {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];

  if (!token) {
    return res.status(401).json({ error: 'Access token required' });
  }

  jwt.verify(token, JWT_SECRET, (err, user) => {
    if (err) {
      return res.status(403).json({ error: 'Invalid or expired token' });
    }
    req.user = user;
    next();
  });
};

// Authorization middleware (Role-based access control)
const authorize = (roles = []) => {
  return (req, res, next) => {
    if (!req.user) {
      return res.status(401).json({ error: 'Unauthorized' });
    }

    if (roles.length && !roles.includes(req.user.role)) {
      return res.status(403).json({ error: 'Insufficient permissions' });
    }

    next();
  };
};

// Validation middleware factory
const validateRequest = (validations) => {
  return async (req, res, next) => {
    await Promise.all(validations.map(validation => validation.run(req)));

    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({
        error: 'Validation failed',
        details: errors.array()
      });
    }

    next();
  };
};

// Caching middleware
const cacheMiddleware = (duration = 300) => {
  return async (req, res, next) => {
    const key = `cache:${req.originalUrl}`;
    
    try {
      const cached = await services.cacheService.get(key);
      if (cached) {
        return res.json(JSON.parse(cached));
      }
      
      // Store original json method
      const originalJson = res.json;
      
      // Override json method to cache response
      res.json = function(data) {
        services.cacheService.set(key, JSON.stringify(data), duration);
        return originalJson.call(this, data);
      };
      
      next();
    } catch (error) {
      logger.error('Cache middleware error:', error);
      next();
    }
  };
};

// Request logging middleware
const requestLogger = (req, res, next) => {
  const startTime = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - startTime;
    logger.info({
      method: req.method,
      url: req.url,
      statusCode: res.statusCode,
      duration: `${duration}ms`,
      userAgent: req.get('User-Agent'),
      ip: req.ip
    });
  });
  
  next();
};

app.use(requestLogger);

// Health check endpoint
app.get('/health', async (req, res) => {
  const health = {
    status: 'OK',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    environment: NODE_ENV,
    version: process.env.npm_package_version || '1.0.0',
    services: {
      database: 'unknown',
      cache: 'unknown'
    }
  };

  try {
    // Check database connection
    await mongooseConnection.db.admin().ping();
    health.services.database = 'connected';
  } catch (error) {
    health.services.database = 'disconnected';
    health.status = 'DEGRADED';
  }

  try {
    // Check Redis connection
    await redisClient.ping();
    health.services.cache = 'connected';
  } catch (error) {
    health.services.cache = 'disconnected';
    health.status = 'DEGRADED';
  }

  const statusCode = health.status === 'OK' ? 200 : 503;
  res.status(statusCode).json(health);
});

// API Routes

// Authentication routes
app.post('/api/auth/register', 
  validateRequest([
    body('username').isLength({ min: 3 }).trim(),
    body('email').isEmail().normalizeEmail(),
    body('password').isLength({ min: 8 })
      .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/)
      .withMessage('Password must contain at least one lowercase letter, uppercase letter, number and special character')
  ]),
  async (req, res, next) => {
    try {
      const { username, email, password } = req.body;
      
      // Check if user already exists
      const existingUser = await services.userService.findByEmail(email);
      if (existingUser) {
        return res.status(400).json({ error: 'User already exists' });
      }

      // Hash password
      const saltRounds = 12;
      const hashedPassword = await bcrypt.hash(password, saltRounds);

      // Create user
      const user = await services.userService.create({
        username,
        email,
        password: hashedPassword,
        role: 'user',
        isVerified: false
      });

      // Generate verification token
      const verificationToken = jwt.sign(
        { userId: user._id, type: 'verification' },
        JWT_SECRET,
        { expiresIn: '24h' }
      );

      // Send verification email (async)
      emailService.sendVerificationEmail(email, verificationToken)
        .catch(error => logger.error('Failed to send verification email:', error));

      res.status(201).json({
        message: 'User registered successfully',
        user: {
          id: user._id,
          username: user.username,
          email: user.email,
          role: user.role
        }
      });
    } catch (error) {
      next(error);
    }
  }
);

app.post('/api/auth/login',
  validateRequest([
    body('email').isEmail().normalizeEmail(),
    body('password').notEmpty()
  ]),
  async (req, res, next) => {
    try {
      const { email, password } = req.body;

      // Find user
      const user = await services.userService.findByEmail(email);
      if (!user) {
        return res.status(401).json({ error: 'Invalid credentials' });
      }

      // Check password
      const isValidPassword = await bcrypt.compare(password, user.password);
      if (!isValidPassword) {
        // Track failed login attempt
        await services.userService.incrementFailedLogins(user._id);
        return res.status(401).json({ error: 'Invalid credentials' });
      }

      // Check if user is locked
      if (user.isLocked) {
        return res.status(423).json({ error: 'Account is locked' });
      }

      // Reset failed login attempts
      await services.userService.resetFailedLogins(user._id);

      // Generate tokens
      const accessToken = jwt.sign(
        { userId: user._id, email: user.email, role: user.role },
        JWT_SECRET,
        { expiresIn: '15m' }
      );

      const refreshToken = jwt.sign(
        { userId: user._id, type: 'refresh' },
        JWT_SECRET,
        { expiresIn: '7d' }
      );

      // Store refresh token in database
      await services.userService.storeRefreshToken(user._id, refreshToken);

      // Update last login
      await services.userService.updateLastLogin(user._id);

      res.json({
        message: 'Login successful',
        accessToken,
        refreshToken,
        user: {
          id: user._id,
          username: user.username,
          email: user.email,
          role: user.role
        }
      });
    } catch (error) {
      next(error);
    }
  }
);

// User routes
app.get('/api/users', 
  authenticateToken, 
  authorize(['admin', 'moderator']), 
  cacheMiddleware(300),
  async (req, res, next) => {
    try {
      const { page = 1, limit = 10, search, role } = req.query;
      const users = await services.userService.findAll({
        page: parseInt(page),
        limit: parseInt(limit),
        search,
        role
      });

      res.json(users);
    } catch (error) {
      next(error);
    }
  }
);

app.get('/api/users/:id', 
  authenticateToken, 
  async (req, res, next) => {
    try {
      const { id } = req.params;
      
      // Users can only view their own profile unless they're admin
      if (req.user.userId !== id && req.user.role !== 'admin') {
        return res.status(403).json({ error: 'Access denied' });
      }

      const user = await services.userService.findById(id);
      if (!user) {
        return res.status(404).json({ error: 'User not found' });
      }

      res.json(user);
    } catch (error) {
      next(error);
    }
  }
);

// Product routes (RESTful API pattern)
app.get('/api/products', cacheMiddleware(600), async (req, res, next) => {
  try {
    const { category, minPrice, maxPrice, search, page = 1, limit = 20 } = req.query;
    
    const filters = {};
    if (category) filters.category = category;
    if (minPrice || maxPrice) {
      filters.price = {};
      if (minPrice) filters.price.$gte = parseFloat(minPrice);
      if (maxPrice) filters.price.$lte = parseFloat(maxPrice);
    }
    if (search) {
      filters.$text = { $search: search };
    }

    const products = await Product.find(filters)
      .skip((page - 1) * limit)
      .limit(parseInt(limit))
      .sort({ createdAt: -1 });

    const total = await Product.countDocuments(filters);

    res.json({
      products,
      pagination: {
        currentPage: parseInt(page),
        totalPages: Math.ceil(total / limit),
        totalItems: total,
        hasNext: page * limit < total,
        hasPrev: page > 1
      }
    });
  } catch (error) {
    next(error);
  }
});

// WebSocket integration pattern
const http = require('http');
const socketIo = require('socket.io');

const server = http.createServer(app);
const io = socketIo(server, {
  cors: {
    origin: corsOptions.origin,
    credentials: true
  }
});

// Socket.io middleware for authentication
io.use(async (socket, next) => {
  try {
    const token = socket.handshake.auth.token;
    if (!token) {
      return next(new Error('Authentication error'));
    }

    const decoded = jwt.verify(token, JWT_SECRET);
    const user = await services.userService.findById(decoded.userId);
    
    if (!user) {
      return next(new Error('User not found'));
    }

    socket.userId = user._id;
    socket.userRole = user.role;
    next();
  } catch (error) {
    next(new Error('Authentication error'));
  }
});

// Socket.io event handlers
io.on('connection', (socket) => {
  logger.info(`User ${socket.userId} connected via WebSocket`);

  // Join user-specific room
  socket.join(`user_${socket.userId}`);

  // Join role-based rooms
  socket.join(`role_${socket.userRole}`);

  // Handle real-time notifications
  socket.on('subscribe_notifications', () => {
    socket.join('notifications');
    logger.info(`User ${socket.userId} subscribed to notifications`);
  });

  // Handle chat messages
  socket.on('send_message', async (data) => {
    try {
      const { message, roomId } = data;
      
      // Validate and sanitize message
      if (!message || message.trim().length === 0) {
        socket.emit('error', { message: 'Message cannot be empty' });
        return;
      }

      // Save message to database
      const savedMessage = await services.chatService.saveMessage({
        userId: socket.userId,
        roomId,
        message: message.trim(),
        timestamp: new Date()
      });

      // Broadcast to room
      io.to(roomId).emit('new_message', savedMessage);
      
    } catch (error) {
      logger.error('Socket message error:', error);
      socket.emit('error', { message: 'Failed to send message' });
    }
  });

  socket.on('disconnect', () => {
    logger.info(`User ${socket.userId} disconnected`);
  });
});

// File upload pattern
const multer = require('multer');
const path = require('path');

// Configure multer for file uploads
const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, 'uploads/');
  },
  filename: (req, file, cb) => {
    const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9);
    cb(null, file.fieldname + '-' + uniqueSuffix + path.extname(file.originalname));
  }
});

const upload = multer({
  storage,
  limits: {
    fileSize: 10 * 1024 * 1024, // 10MB
    files: 5
  },
  fileFilter: (req, file, cb) => {
    const allowedTypes = /jpeg|jpg|png|gif|pdf|doc|docx/;
    const extname = allowedTypes.test(path.extname(file.originalname).toLowerCase());
    const mimetype = allowedTypes.test(file.mimetype);

    if (mimetype && extname) {
      return cb(null, true);
    } else {
      cb(new Error('Invalid file type'));
    }
  }
});

app.post('/api/upload', 
  authenticateToken, 
  upload.array('files', 5), 
  async (req, res, next) => {
    try {
      const files = req.files.map(file => ({
        filename: file.filename,
        originalName: file.originalname,
        mimetype: file.mimetype,
        size: file.size,
        uploadedBy: req.user.userId,
        uploadedAt: new Date()
      }));

      res.json({
        message: 'Files uploaded successfully',
        files
      });
    } catch (error) {
      next(error);
    }
  }
);

// Async/await error handling pattern
const asyncHandler = (fn) => (req, res, next) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};

// Background job pattern with Bull Queue
const Queue = require('bull');
const emailQueue = new Queue('email processing', REDIS_URL);

emailQueue.process('send-email', async (job) => {
  const { to, subject, template, data } = job.data;
  
  try {
    await emailService.sendTemplateEmail(to, subject, template, data);
    logger.info(`Email sent successfully to ${to}`);
  } catch (error) {
    logger.error(`Failed to send email to ${to}:`, error);
    throw error;
  }
});

// Schedule email job
app.post('/api/send-email', 
  authenticateToken, 
  authorize(['admin']),
  async (req, res, next) => {
    try {
      const { to, subject, template, data } = req.body;
      
      await emailQueue.add('send-email', {
        to,
        subject,
        template,
        data
      }, {
        attempts: 3,
        backoff: 'exponential',
        delay: 5000
      });

      res.json({ message: 'Email queued successfully' });
    } catch (error) {
      next(error);
    }
  }
);

// Graceful shutdown pattern
const gracefulShutdown = () => {
  logger.info('Starting graceful shutdown...');
  
  server.close(() => {
    logger.info('HTTP server closed');
    
    // Close database connections
    mongooseConnection.close(() => {
      logger.info('MongoDB connection closed');
    });

    redisClient.quit(() => {
      logger.info('Redis connection closed');
    });

    // Close email queue
    emailQueue.close().then(() => {
      logger.info('Email queue closed');
      process.exit(0);
    });
  });

  // Force close after 30 seconds
  setTimeout(() => {
    logger.error('Could not close connections in time, forcefully shutting down');
    process.exit(1);
  }, 30000);
};

// Handle shutdown signals
process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);

// Global error handler
app.use(errorHandler);

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({
    error: 'Route not found',
    message: `Cannot ${req.method} ${req.originalUrl}`
  });
});

// Start server
const startServer = async () => {
  try {
    // Connect to databases
    await mongooseConnection.asPromise();
    await redisClient.connect();
    
    logger.info('Database connections established');

    server.listen(PORT, () => {
      logger.info(`Server running on port ${PORT} in ${NODE_ENV} mode`);
      logger.info(`API documentation available at http://localhost:${PORT}/api-docs`);
    });
  } catch (error) {
    logger.error('Failed to start server:', error);
    process.exit(1);
  }
};

if (require.main === module) {
  startServer();
}

module.exports = { app, server, io }; 