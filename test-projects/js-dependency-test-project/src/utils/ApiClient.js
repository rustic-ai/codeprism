/**
 * ApiClient - HTTP client wrapper for API communication
 */

import axios from 'axios';
import { Logger } from './Logger';

export class ApiClient {
  constructor(config = {}) {
    this.logger = new Logger('ApiClient');
    this.defaultConfig = {
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
      ...config,
    };
    
    this.client = this.createClient();
  }
  
  createClient() {
    const client = axios.create(this.defaultConfig);
    
    // Request interceptor
    client.interceptors.request.use(
      (config) => {
        this.logger.debug('API Request', {
          method: config.method?.toUpperCase(),
          url: config.url,
          params: config.params,
        });
        return config;
      },
      (error) => {
        this.logger.error('Request interceptor error', error);
        return Promise.reject(error);
      }
    );
    
    // Response interceptor
    client.interceptors.response.use(
      (response) => {
        this.logger.debug('API Response', {
          status: response.status,
          url: response.config.url,
        });
        return response.data;
      },
      (error) => {
        this.logger.error('API Error', {
          status: error.response?.status,
          message: error.message,
          url: error.config?.url,
        });
        return Promise.reject(error);
      }
    );
    
    return client;
  }
  
  async configure(config) {
    Object.assign(this.defaultConfig, config);
    this.client = this.createClient();
  }
  
  async get(url, config = {}) {
    return this.client.get(url, config);
  }
  
  async post(url, data, config = {}) {
    return this.client.post(url, data, config);
  }
  
  async put(url, data, config = {}) {
    return this.client.put(url, data, config);
  }
  
  async delete(url, config = {}) {
    return this.client.delete(url, config);
  }
} 