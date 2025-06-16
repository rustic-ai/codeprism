/**
 * UserService - Service layer for user operations
 * Demonstrates service layer patterns and dependency injection
 */

import { ApiClient } from '../utils/ApiClient';
import { Logger } from '../utils/Logger';
import { CacheManager } from '../utils/CacheManager';

export class UserService {
  constructor(dependencies = {}) {
    this.apiClient = dependencies.apiClient || new ApiClient();
    this.authProvider = dependencies.authProvider;
    this.config = dependencies.config;
    
    this.logger = new Logger('UserService');
    this.cache = new CacheManager('users');
    
    this.endpoints = {
      users: '/api/users',
      userById: '/api/users/:id',
      userSearch: '/api/users/search',
      userProfile: '/api/users/:id/profile',
    };
    
    this.initialized = false;
  }
  
  async initialize() {
    if (this.initialized) return;
    
    this.logger.info('Initializing UserService');
    
    // Setup API client
    if (this.config) {
      await this.apiClient.configure({
        baseURL: this.config.get('api.baseUrl'),
        timeout: this.config.get('api.timeout', 10000),
        headers: {
          'Content-Type': 'application/json',
        },
      });
    }
    
    this.initialized = true;
    this.logger.info('UserService initialized successfully');
  }
  
  setApiEndpoints(endpoints) {
    Object.assign(this.endpoints, endpoints);
  }
  
  async getUsers(filters = {}, options = {}) {
    const cacheKey = `users:${JSON.stringify(filters)}`;
    
    // Check cache first unless force refresh
    if (!options.forceRefresh) {
      const cached = await this.cache.get(cacheKey);
      if (cached) {
        this.logger.debug('Returning cached users');
        return cached;
      }
    }
    
    this.logger.info('Fetching users from API', { filters });
    
    const response = await this.apiClient.get(this.endpoints.users, {
      params: filters,
      ...options,
    });
    
    // Cache successful response
    await this.cache.set(cacheKey, response, { ttl: 300 }); // 5 minutes
    
    return response;
  }
  
  async getUserById(userId, options = {}) {
    const cacheKey = `user:${userId}`;
    
    if (!options.forceRefresh) {
      const cached = await this.cache.get(cacheKey);
      if (cached) {
        return cached;
      }
    }
    
    this.logger.info('Fetching user by ID', { userId });
    
    const endpoint = this.endpoints.userById.replace(':id', userId);
    const response = await this.apiClient.get(endpoint, options);
    
    await this.cache.set(cacheKey, response, { ttl: 600 }); // 10 minutes
    
    return response;
  }
  
  async searchUsers(searchParams) {
    this.logger.info('Searching users', { searchParams });
    
    return await this.apiClient.get(this.endpoints.userSearch, {
      params: searchParams,
    });
  }
  
  async createUser(userData) {
    this.logger.info('Creating new user', { userData: { ...userData, password: '[REDACTED]' } });
    
    const response = await this.apiClient.post(this.endpoints.users, userData);
    
    // Invalidate cache
    await this.cache.invalidatePattern('users:*');
    
    return response;
  }
  
  async updateUser(userId, userData) {
    this.logger.info('Updating user', { userId, userData: { ...userData, password: '[REDACTED]' } });
    
    const endpoint = this.endpoints.userById.replace(':id', userId);
    const response = await this.apiClient.put(endpoint, userData);
    
    // Invalidate related cache entries
    await this.cache.invalidate(`user:${userId}`);
    await this.cache.invalidatePattern('users:*');
    
    return response;
  }
  
  async deleteUser(userId) {
    this.logger.warn('Deleting user', { userId });
    
    const endpoint = this.endpoints.userById.replace(':id', userId);
    const response = await this.apiClient.delete(endpoint);
    
    // Invalidate cache
    await this.cache.invalidate(`user:${userId}`);
    await this.cache.invalidatePattern('users:*');
    
    return response;
  }
  
  async getUserProfile(userId) {
    const endpoint = this.endpoints.userProfile.replace(':id', userId);
    return await this.apiClient.get(endpoint);
  }
  
  async updateUserProfile(userId, profileData) {
    const endpoint = this.endpoints.userProfile.replace(':id', userId);
    const response = await this.apiClient.put(endpoint, profileData);
    
    // Invalidate cache
    await this.cache.invalidate(`user:${userId}`);
    
    return response;
  }
} 