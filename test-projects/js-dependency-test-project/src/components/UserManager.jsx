/**
 * UserManager Component - Core React component for dependency testing
 * This file demonstrates realistic JavaScript dependency patterns similar to the Python Agent class issue.
 */

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import PropTypes from 'prop-types';
import { connect } from 'react-redux';
import { debounce } from 'lodash';
import axios from 'axios';
import moment from 'moment';

// Internal project imports
import { UserService } from '../services/UserService';
import { AuthProvider } from '../auth/AuthProvider';
import { ApiClient } from '../utils/ApiClient';
import { Logger } from '../utils/Logger';
import { Config } from '../config/Config';
import { MessageBus } from '../messaging/MessageBus';

// Component imports
import UserList from './UserList';
import UserForm from './UserForm';
import LoadingSpinner from './LoadingSpinner';
import ErrorBoundary from './ErrorBoundary';

// Constants and enums
const USER_STATES = {
  IDLE: 'idle',
  LOADING: 'loading',
  LOADED: 'loaded',
  ERROR: 'error',
  SAVING: 'saving',
};

const API_ENDPOINTS = {
  USERS: '/api/users',
  USER_PROFILE: '/api/user/profile',
  USER_PERMISSIONS: '/api/user/permissions',
};

/**
 * UserManager - Main component demonstrating complex dependency usage patterns
 * 
 * This component shows realistic usage of:
 * - External dependencies (react, lodash, axios, moment)
 * - Internal project modules
 * - Complex state management
 * - API integration patterns
 * - Performance optimizations
 */
class UserManager extends React.Component {
  constructor(props) {
    super(props);
    
    this.state = {
      users: [],
      selectedUser: null,
      searchQuery: '',
      status: USER_STATES.IDLE,
      error: null,
      isFormVisible: false,
      lastUpdated: null,
    };
    
    // Service dependencies
    this.userService = new UserService({
      apiClient: new ApiClient(),
      authProvider: new AuthProvider(),
      config: Config.getInstance(),
    });
    
    this.logger = new Logger('UserManager');
    this.messageBus = MessageBus.getInstance();
    
    // Debounced search handler
    this.debouncedSearch = debounce(this.performSearch.bind(this), 300);
    
    // Bind methods
    this.handleUserSelect = this.handleUserSelect.bind(this);
    this.handleUserSave = this.handleUserSave.bind(this);
    this.handleUserDelete = this.handleUserDelete.bind(this);
    this.handleSearchChange = this.handleSearchChange.bind(this);
    this.handleFormToggle = this.handleFormToggle.bind(this);
    this.handleRefresh = this.handleRefresh.bind(this);
    
    // Message bus subscriptions
    this.subscriptions = [];
  }
  
  async componentDidMount() {
    this.logger.info('UserManager component mounted');
    
    try {
      // Initialize component
      await this.initializeComponent();
      
      // Set up message bus subscriptions
      this.setupMessageSubscriptions();
      
      // Load initial data
      await this.loadUsers();
      
    } catch (error) {
      this.logger.error('Failed to initialize UserManager', error);
      this.setState({ 
        status: USER_STATES.ERROR, 
        error: error.message 
      });
    }
  }
  
  componentDidUpdate(prevProps, prevState) {
    // React to prop changes
    if (prevProps.currentUser !== this.props.currentUser) {
      this.handleCurrentUserChange();
    }
    
    // Update search when query changes
    if (prevState.searchQuery !== this.state.searchQuery) {
      this.debouncedSearch(this.state.searchQuery);
    }
  }
  
  componentWillUnmount() {
    this.logger.info('UserManager component unmounting');
    
    // Cleanup subscriptions
    this.subscriptions.forEach(subscription => {
      this.messageBus.unsubscribe(subscription.id);
    });
    
    // Cancel pending requests
    if (this.cancelToken) {
      this.cancelToken.cancel('Component unmounted');
    }
    
    // Cleanup debounced function
    this.debouncedSearch.cancel();
  }
  
  async initializeComponent() {
    this.logger.debug('Initializing UserManager component');
    
    // Initialize services
    await this.userService.initialize();
    
    // Setup API client configuration
    const config = Config.getInstance();
    this.userService.setApiEndpoints({
      baseUrl: config.get('api.baseUrl'),
      timeout: config.get('api.timeout', 5000),
      retries: config.get('api.retries', 3),
    });
    
    // Create cancel token for requests
    this.cancelToken = axios.CancelToken.source();
  }
  
  setupMessageSubscriptions() {
    // Subscribe to user-related events
    const userUpdatedSub = this.messageBus.subscribe('user.updated', (userData) => {
      this.handleUserUpdatedEvent(userData);
    });
    
    const userDeletedSub = this.messageBus.subscribe('user.deleted', (userId) => {
      this.handleUserDeletedEvent(userId);
    });
    
    const authChangedSub = this.messageBus.subscribe('auth.changed', (authData) => {
      this.handleAuthChangedEvent(authData);
    });
    
    this.subscriptions.push(userUpdatedSub, userDeletedSub, authChangedSub);
  }
  
  async loadUsers(refresh = false) {
    this.logger.info('Loading users', { refresh });
    
    this.setState({ status: USER_STATES.LOADING });
    
    try {
      const filters = {
        active: true,
        orderBy: 'name',
        limit: 100,
      };
      
      const response = await this.userService.getUsers(filters, {
        cancelToken: this.cancelToken.token,
        forceRefresh: refresh,
      });
      
      this.setState({
        users: response.data || [],
        status: USER_STATES.LOADED,
        error: null,
        lastUpdated: moment().toISOString(),
      });
      
      this.logger.info(`Loaded ${response.data?.length || 0} users`);
      
    } catch (error) {
      if (!axios.isCancel(error)) {
        this.logger.error('Failed to load users', error);
        this.setState({
          status: USER_STATES.ERROR,
          error: error.message,
        });
      }
    }
  }
  
  async performSearch(query) {
    if (!query || query.trim().length < 2) {
      await this.loadUsers();
      return;
    }
    
    this.logger.debug('Performing user search', { query });
    
    this.setState({ status: USER_STATES.LOADING });
    
    try {
      const searchResults = await this.userService.searchUsers({
        query: query.trim(),
        fields: ['name', 'email', 'username'],
        limit: 50,
      });
      
      this.setState({
        users: searchResults.data || [],
        status: USER_STATES.LOADED,
        error: null,
      });
      
    } catch (error) {
      this.logger.error('Search failed', error);
      this.setState({
        status: USER_STATES.ERROR,
        error: error.message,
      });
    }
  }
  
  handleUserSelect(user) {
    this.logger.debug('User selected', { userId: user.id });
    
    this.setState({ 
      selectedUser: user,
      isFormVisible: false,
    });
    
    // Publish selection event
    this.messageBus.publish('user.selected', user);
  }
  
  async handleUserSave(userData) {
    this.logger.info('Saving user', { userData });
    
    this.setState({ status: USER_STATES.SAVING });
    
    try {
      let savedUser;
      
      if (userData.id) {
        // Update existing user
        savedUser = await this.userService.updateUser(userData.id, userData);
        this.logger.info('User updated successfully', { userId: userData.id });
      } else {
        // Create new user
        savedUser = await this.userService.createUser(userData);
        this.logger.info('User created successfully', { userId: savedUser.id });
      }
      
      // Refresh user list
      await this.loadUsers(true);
      
      // Select the saved user
      this.setState({
        selectedUser: savedUser,
        isFormVisible: false,
        status: USER_STATES.LOADED,
      });
      
      // Publish save event
      this.messageBus.publish('user.saved', savedUser);
      
    } catch (error) {
      this.logger.error('Failed to save user', error);
      this.setState({
        status: USER_STATES.ERROR,
        error: error.message,
      });
    }
  }
  
  async handleUserDelete(userId) {
    if (!window.confirm('Are you sure you want to delete this user?')) {
      return;
    }
    
    this.logger.warn('Deleting user', { userId });
    
    try {
      await this.userService.deleteUser(userId);
      
      // Remove from local state
      this.setState(prevState => ({
        users: prevState.users.filter(user => user.id !== userId),
        selectedUser: prevState.selectedUser?.id === userId ? null : prevState.selectedUser,
      }));
      
      this.logger.info('User deleted successfully', { userId });
      
      // Publish delete event
      this.messageBus.publish('user.deleted', userId);
      
    } catch (error) {
      this.logger.error('Failed to delete user', error);
      this.setState({
        status: USER_STATES.ERROR,
        error: error.message,
      });
    }
  }
  
  handleSearchChange(event) {
    const query = event.target.value;
    this.setState({ searchQuery: query });
  }
  
  handleFormToggle() {
    this.setState(prevState => ({
      isFormVisible: !prevState.isFormVisible,
      selectedUser: prevState.isFormVisible ? prevState.selectedUser : null,
    }));
  }
  
  async handleRefresh() {
    await this.loadUsers(true);
  }
  
  handleCurrentUserChange() {
    this.logger.debug('Current user changed', { currentUser: this.props.currentUser });
    // Reload data if current user permissions changed
    if (this.props.currentUser) {
      this.loadUsers(true);
    }
  }
  
  handleUserUpdatedEvent(userData) {
    this.setState(prevState => ({
      users: prevState.users.map(user => 
        user.id === userData.id ? { ...user, ...userData } : user
      ),
    }));
  }
  
  handleUserDeletedEvent(userId) {
    this.setState(prevState => ({
      users: prevState.users.filter(user => user.id !== userId),
      selectedUser: prevState.selectedUser?.id === userId ? null : prevState.selectedUser,
    }));
  }
  
  handleAuthChangedEvent(authData) {
    if (!authData.isAuthenticated) {
      // Clear sensitive data on logout
      this.setState({
        users: [],
        selectedUser: null,
        status: USER_STATES.IDLE,
      });
    }
  }
  
  render() {
    const { 
      users, 
      selectedUser, 
      searchQuery, 
      status, 
      error, 
      isFormVisible, 
      lastUpdated 
    } = this.state;
    
    const { className, theme, permissions } = this.props;
    
    const isLoading = status === USER_STATES.LOADING || status === USER_STATES.SAVING;
    const hasError = status === USER_STATES.ERROR;
    const canCreateUser = permissions?.includes('user:create');
    const canEditUser = permissions?.includes('user:edit');
    const canDeleteUser = permissions?.includes('user:delete');
    
    return (
      <ErrorBoundary onError={(error) => this.logger.error('UserManager render error', error)}>
        <div className={`user-manager ${className || ''} theme-${theme || 'default'}`}>
          {/* Header */}
          <div className="user-manager__header">
            <h2>User Management</h2>
            {lastUpdated && (
              <span className="last-updated">
                Last updated: {moment(lastUpdated).fromNow()}
              </span>
            )}
          </div>
          
          {/* Controls */}
          <div className="user-manager__controls">
            <div className="search-container">
              <input
                type="text"
                placeholder="Search users..."
                value={searchQuery}
                onChange={this.handleSearchChange}
                disabled={isLoading}
                className="search-input"
              />
            </div>
            
            <div className="action-buttons">
              <button
                onClick={this.handleRefresh}
                disabled={isLoading}
                className="btn btn-secondary"
              >
                Refresh
              </button>
              
              {canCreateUser && (
                <button
                  onClick={this.handleFormToggle}
                  disabled={isLoading}
                  className="btn btn-primary"
                >
                  {isFormVisible ? 'Cancel' : 'Add User'}
                </button>
              )}
            </div>
          </div>
          
          {/* Loading Spinner */}
          {isLoading && <LoadingSpinner />}
          
          {/* Error Display */}
          {hasError && (
            <div className="error-message">
              <strong>Error:</strong> {error}
              <button onClick={() => this.setState({ status: USER_STATES.IDLE, error: null })}>
                Dismiss
              </button>
            </div>
          )}
          
          {/* Main Content */}
          <div className="user-manager__content">
            {/* User Form */}
            {isFormVisible && (
              <UserForm
                user={selectedUser}
                onSave={this.handleUserSave}
                onCancel={this.handleFormToggle}
                permissions={{ canEdit: canEditUser }}
              />
            )}
            
            {/* User List */}
            <UserList
              users={users}
              selectedUser={selectedUser}
              onUserSelect={this.handleUserSelect}
              onUserDelete={canDeleteUser ? this.handleUserDelete : null}
              onUserEdit={canEditUser ? (user) => {
                this.setState({ selectedUser: user, isFormVisible: true });
              } : null}
              loading={isLoading}
              searchQuery={searchQuery}
            />
          </div>
        </div>
      </ErrorBoundary>
    );
  }
}

UserManager.propTypes = {
  currentUser: PropTypes.object,
  permissions: PropTypes.arrayOf(PropTypes.string),
  className: PropTypes.string,
  theme: PropTypes.oneOf(['default', 'dark', 'light']),
  onUserChange: PropTypes.func,
};

UserManager.defaultProps = {
  currentUser: null,
  permissions: [],
  className: '',
  theme: 'default',
  onUserChange: () => {},
};

// Redux connection
const mapStateToProps = (state) => ({
  currentUser: state.auth.currentUser,
  permissions: state.auth.permissions,
  theme: state.ui.theme,
});

const mapDispatchToProps = (dispatch) => ({
  onUserChange: (user) => dispatch({ type: 'USER_CHANGED', payload: user }),
});

export default connect(mapStateToProps, mapDispatchToProps)(UserManager); 