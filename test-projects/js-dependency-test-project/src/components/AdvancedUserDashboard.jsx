/**
 * Advanced User Dashboard Component
 * Demonstrates React hooks, context, patterns, performance optimizations, and modern React features
 */

import React, { 
  useState, 
  useEffect, 
  useCallback, 
  useMemo, 
  useReducer, 
  useRef, 
  useContext, 
  createContext, 
  memo, 
  lazy, 
  Suspense,
  forwardRef,
  useImperativeHandle,
  useLayoutEffect,
  useDeferredValue,
  useTransition,
  useId,
  useSyncExternalStore
} from 'react';
import PropTypes from 'prop-types';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useForm, Controller } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';

// Context setup
const UserContext = createContext();
const ThemeContext = createContext();
const NotificationContext = createContext();

// Custom hooks
const useLocalStorage = (key, initialValue) => {
  const [storedValue, setStoredValue] = useState(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      console.error(`Error reading localStorage key "${key}":`, error);
      return initialValue;
    }
  });

  const setValue = useCallback((value) => {
    try {
      const valueToStore = value instanceof Function ? value(storedValue) : value;
      setStoredValue(valueToStore);
      window.localStorage.setItem(key, JSON.stringify(valueToStore));
    } catch (error) {
      console.error(`Error setting localStorage key "${key}":`, error);
    }
  }, [key, storedValue]);

  return [storedValue, setValue];
};

const useDebounce = (value, delay) => {
  const [debouncedValue, setDebouncedValue] = useState(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
};

const useIntersectionObserver = (options = {}) => {
  const [isIntersecting, setIsIntersecting] = useState(false);
  const ref = useRef();

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsIntersecting(entry.isIntersecting);
      },
      options
    );

    if (ref.current) {
      observer.observe(ref.current);
    }

    return () => {
      observer.disconnect();
    };
  }, [options]);

  return [ref, isIntersecting];
};

const useWebSocket = (url) => {
  const [socket, setSocket] = useState(null);
  const [lastMessage, setLastMessage] = useState(null);
  const [readyState, setReadyState] = useState(WebSocket.CONNECTING);

  useEffect(() => {
    const ws = new WebSocket(url);
    
    ws.onopen = () => setReadyState(WebSocket.OPEN);
    ws.onclose = () => setReadyState(WebSocket.CLOSED);
    ws.onerror = () => setReadyState(WebSocket.CLOSED);
    ws.onmessage = (event) => {
      setLastMessage(JSON.parse(event.data));
    };

    setSocket(ws);

    return () => {
      ws.close();
    };
  }, [url]);

  const sendMessage = useCallback((data) => {
    if (socket && readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(data));
    }
  }, [socket, readyState]);

  return { socket, lastMessage, readyState, sendMessage };
};

// External store for global state management
const createStore = (initialState) => {
  let state = initialState;
  const listeners = new Set();

  const getState = () => state;

  const setState = (newState) => {
    state = { ...state, ...newState };
    listeners.forEach(listener => listener());
  };

  const subscribe = (listener) => {
    listeners.add(listener);
    return () => listeners.delete(listener);
  };

  return { getState, setState, subscribe };
};

const userStore = createStore({
  users: [],
  selectedUser: null,
  loading: false,
  error: null
});

// Reducer for complex state management
const dashboardReducer = (state, action) => {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
    case 'SET_USERS':
      return { ...state, users: action.payload, loading: false };
    case 'ADD_USER':
      return { ...state, users: [...state.users, action.payload] };
    case 'UPDATE_USER':
      return {
        ...state,
        users: state.users.map(user =>
          user.id === action.payload.id ? { ...user, ...action.payload } : user
        )
      };
    case 'DELETE_USER':
      return {
        ...state,
        users: state.users.filter(user => user.id !== action.payload)
      };
    case 'SET_FILTER':
      return { ...state, filter: action.payload };
    case 'SET_SORT':
      return { ...state, sort: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
    default:
      return state;
  }
};

// Validation schema
const userSchema = yup.object().shape({
  name: yup.string().required('Name is required').min(2, 'Name must be at least 2 characters'),
  email: yup.string().email('Invalid email format').required('Email is required'),
  age: yup.number().positive('Age must be positive').integer('Age must be an integer').required('Age is required'),
  role: yup.string().oneOf(['admin', 'user', 'moderator'], 'Invalid role').required('Role is required')
});

// Lazy loaded components
const UserAnalytics = lazy(() => import('./UserAnalytics'));
const UserReports = lazy(() => import('./UserReports'));
const UserSettings = lazy(() => import('./UserSettings'));

// Memoized components
const UserCard = memo(({ user, onEdit, onDelete, isSelected }) => {
  const theme = useContext(ThemeContext);
  const { addNotification } = useContext(NotificationContext);

  const handleEdit = useCallback(() => {
    onEdit(user);
    addNotification(`Editing user: ${user.name}`, 'info');
  }, [user, onEdit, addNotification]);

  const handleDelete = useCallback(() => {
    if (window.confirm(`Are you sure you want to delete ${user.name}?`)) {
      onDelete(user.id);
      addNotification(`User ${user.name} deleted`, 'success');
    }
  }, [user, onDelete, addNotification]);

  return (
    <div 
      className={`user-card ${isSelected ? 'selected' : ''} ${theme.mode}`}
      role="article"
      aria-label={`User card for ${user.name}`}
    >
      <div className="user-avatar">
        <img 
          src={user.avatar || '/default-avatar.png'} 
          alt={`${user.name}'s avatar`}
          loading="lazy"
        />
      </div>
      <div className="user-info">
        <h3>{user.name}</h3>
        <p>{user.email}</p>
        <span className={`role-badge ${user.role}`}>{user.role}</span>
      </div>
      <div className="user-actions">
        <button onClick={handleEdit} aria-label={`Edit ${user.name}`}>
          Edit
        </button>
        <button onClick={handleDelete} aria-label={`Delete ${user.name}`}>
          Delete
        </button>
      </div>
    </div>
  );
});

UserCard.propTypes = {
  user: PropTypes.shape({
    id: PropTypes.oneOfType([PropTypes.string, PropTypes.number]).isRequired,
    name: PropTypes.string.isRequired,
    email: PropTypes.string.isRequired,
    avatar: PropTypes.string,
    role: PropTypes.oneOf(['admin', 'user', 'moderator']).isRequired
  }).isRequired,
  onEdit: PropTypes.func.isRequired,
  onDelete: PropTypes.func.isRequired,
  isSelected: PropTypes.bool
};

UserCard.defaultProps = {
  isSelected: false
};

// Forward ref component
const SearchInput = forwardRef(({ onSearch, placeholder, ...props }, ref) => {
  const [value, setValue] = useState('');
  const debouncedValue = useDebounce(value, 300);
  const inputId = useId();

  useEffect(() => {
    onSearch(debouncedValue);
  }, [debouncedValue, onSearch]);

  useImperativeHandle(ref, () => ({
    clear: () => setValue(''),
    focus: () => ref.current?.focus(),
    getValue: () => value
  }));

  return (
    <div className="search-input-container">
      <label htmlFor={inputId} className="sr-only">
        {placeholder}
      </label>
      <input
        ref={ref}
        id={inputId}
        type="text"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder={placeholder}
        {...props}
      />
      {value && (
        <button 
          onClick={() => setValue('')}
          aria-label="Clear search"
          className="clear-button"
        >
          ×
        </button>
      )}
    </div>
  );
});

SearchInput.propTypes = {
  onSearch: PropTypes.func.isRequired,
  placeholder: PropTypes.string
};

SearchInput.defaultProps = {
  placeholder: 'Search...'
};

// Main component
const AdvancedUserDashboard = () => {
  // State management with useReducer
  const [state, dispatch] = useReducer(dashboardReducer, {
    users: [],
    loading: false,
    error: null,
    filter: '',
    sort: 'name'
  });

  // External store state
  const storeState = useSyncExternalStore(
    userStore.subscribe,
    userStore.getState,
    userStore.getState
  );

  // Local state
  const [selectedUsers, setSelectedUsers] = useState(new Set());
  const [viewMode, setViewMode] = useLocalStorage('viewMode', 'grid');
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingUser, setEditingUser] = useState(null);

  // Refs
  const searchRef = useRef();
  const containerRef = useRef();
  
  // Transition and deferred values for performance
  const [isPending, startTransition] = useTransition();
  const deferredFilter = useDeferredValue(state.filter);

  // Custom hooks
  const [settingsRef, isSettingsVisible] = useIntersectionObserver({
    threshold: 0.1
  });

  const { lastMessage, sendMessage } = useWebSocket('ws://localhost:8080/users');

  // React Query
  const queryClient = useQueryClient();

  const {
    data: users = [],
    isLoading,
    error,
    refetch
  } = useQuery({
    queryKey: ['users', deferredFilter, state.sort],
    queryFn: async () => {
      const response = await fetch(`/api/users?filter=${deferredFilter}&sort=${state.sort}`);
      if (!response.ok) throw new Error('Failed to fetch users');
      return response.json();
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
    cacheTime: 10 * 60 * 1000 // 10 minutes
  });

  const createUserMutation = useMutation({
    mutationFn: async (userData) => {
      const response = await fetch('/api/users', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData)
      });
      if (!response.ok) throw new Error('Failed to create user');
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries(['users']);
      setIsFormOpen(false);
    }
  });

  const updateUserMutation = useMutation({
    mutationFn: async ({ id, ...userData }) => {
      const response = await fetch(`/api/users/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData)
      });
      if (!response.ok) throw new Error('Failed to update user');
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries(['users']);
      setEditingUser(null);
      setIsFormOpen(false);
    }
  });

  const deleteUserMutation = useMutation({
    mutationFn: async (userId) => {
      const response = await fetch(`/api/users/${userId}`, {
        method: 'DELETE'
      });
      if (!response.ok) throw new Error('Failed to delete user');
    },
    onSuccess: () => {
      queryClient.invalidateQueries(['users']);
    }
  });

  // Form handling
  const {
    control,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting }
  } = useForm({
    resolver: yupResolver(userSchema),
    defaultValues: {
      name: '',
      email: '',
      age: '',
      role: 'user'
    }
  });

  // Effects
  useEffect(() => {
    if (lastMessage) {
      switch (lastMessage.type) {
        case 'USER_CREATED':
          queryClient.invalidateQueries(['users']);
          break;
        case 'USER_UPDATED':
          queryClient.setQueryData(['users'], (old) =>
            old?.map(user => 
              user.id === lastMessage.data.id ? lastMessage.data : user
            )
          );
          break;
        case 'USER_DELETED':
          queryClient.setQueryData(['users'], (old) =>
            old?.filter(user => user.id !== lastMessage.data.id)
          );
          break;
        default:
          break;
      }
    }
  }, [lastMessage, queryClient]);

  useEffect(() => {
    if (editingUser) {
      reset(editingUser);
      setIsFormOpen(true);
    }
  }, [editingUser, reset]);

  // Layout effect for DOM measurements
  useLayoutEffect(() => {
    if (containerRef.current) {
      const { height } = containerRef.current.getBoundingClientRect();
      document.documentElement.style.setProperty('--dashboard-height', `${height}px`);
    }
  }, [users.length]);

  // Memoized values
  const filteredUsers = useMemo(() => {
    if (!deferredFilter) return users;
    return users.filter(user =>
      user.name.toLowerCase().includes(deferredFilter.toLowerCase()) ||
      user.email.toLowerCase().includes(deferredFilter.toLowerCase())
    );
  }, [users, deferredFilter]);

  const sortedUsers = useMemo(() => {
    return [...filteredUsers].sort((a, b) => {
      switch (state.sort) {
        case 'name':
          return a.name.localeCompare(b.name);
        case 'email':
          return a.email.localeCompare(b.email);
        case 'role':
          return a.role.localeCompare(b.role);
        default:
          return 0;
      }
    });
  }, [filteredUsers, state.sort]);

  const selectedUsersArray = useMemo(() => {
    return sortedUsers.filter(user => selectedUsers.has(user.id));
  }, [sortedUsers, selectedUsers]);

  // Callbacks
  const handleSearch = useCallback((searchTerm) => {
    startTransition(() => {
      dispatch({ type: 'SET_FILTER', payload: searchTerm });
    });
  }, []);

  const handleSort = useCallback((sortBy) => {
    dispatch({ type: 'SET_SORT', payload: sortBy });
  }, []);

  const handleUserSelect = useCallback((userId, isSelected) => {
    setSelectedUsers(prev => {
      const newSet = new Set(prev);
      if (isSelected) {
        newSet.add(userId);
      } else {
        newSet.delete(userId);
      }
      return newSet;
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    const allUserIds = new Set(sortedUsers.map(user => user.id));
    setSelectedUsers(selectedUsers.size === sortedUsers.length ? new Set() : allUserIds);
  }, [sortedUsers, selectedUsers.size]);

  const handleBulkDelete = useCallback(async () => {
    if (selectedUsersArray.length === 0) return;
    
    const confirmed = window.confirm(
      `Are you sure you want to delete ${selectedUsersArray.length} users?`
    );
    
    if (confirmed) {
      try {
        await Promise.all(
          selectedUsersArray.map(user => deleteUserMutation.mutateAsync(user.id))
        );
        setSelectedUsers(new Set());
      } catch (error) {
        console.error('Bulk delete failed:', error);
      }
    }
  }, [selectedUsersArray, deleteUserMutation]);

  const handleFormSubmit = useCallback(async (data) => {
    try {
      if (editingUser) {
        await updateUserMutation.mutateAsync({ ...data, id: editingUser.id });
      } else {
        await createUserMutation.mutateAsync(data);
      }
    } catch (error) {
      console.error('Form submission failed:', error);
    }
  }, [editingUser, updateUserMutation, createUserMutation]);

  const handleEditUser = useCallback((user) => {
    setEditingUser(user);
  }, []);

  const handleDeleteUser = useCallback(async (userId) => {
    try {
      await deleteUserMutation.mutateAsync(userId);
    } catch (error) {
      console.error('Delete failed:', error);
    }
  }, [deleteUserMutation]);

  // Context values
  const themeContextValue = useMemo(() => ({
    mode: 'light',
    colors: {
      primary: '#007bff',
      secondary: '#6c757d',
      success: '#28a745',
      danger: '#dc3545'
    }
  }), []);

  const notificationContextValue = useMemo(() => ({
    addNotification: (message, type) => {
      console.log(`${type.toUpperCase()}: ${message}`);
    }
  }), []);

  const userContextValue = useMemo(() => ({
    currentUser: { id: 1, name: 'Admin', role: 'admin' },
    permissions: ['read', 'write', 'delete']
  }), []);

  // Render loading state
  if (isLoading) {
    return (
      <div className="dashboard-loading" role="status" aria-label="Loading users">
        <div className="spinner" />
        <p>Loading users...</p>
      </div>
    );
  }

  // Render error state
  if (error) {
    return (
      <div className="dashboard-error" role="alert">
        <h2>Error loading users</h2>
        <p>{error.message}</p>
        <button onClick={() => refetch()}>Retry</button>
      </div>
    );
  }

  return (
    <UserContext.Provider value={userContextValue}>
      <ThemeContext.Provider value={themeContextValue}>
        <NotificationContext.Provider value={notificationContextValue}>
          <div ref={containerRef} className="advanced-user-dashboard">
            {/* Header */}
            <header className="dashboard-header">
              <h1>User Management Dashboard</h1>
              <div className="header-actions">
                <SearchInput
                  ref={searchRef}
                  onSearch={handleSearch}
                  placeholder="Search users..."
                />
                <select 
                  value={state.sort} 
                  onChange={(e) => handleSort(e.target.value)}
                  aria-label="Sort users"
                >
                  <option value="name">Sort by Name</option>
                  <option value="email">Sort by Email</option>
                  <option value="role">Sort by Role</option>
                </select>
                <button
                  onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}
                  aria-label={`Switch to ${viewMode === 'grid' ? 'list' : 'grid'} view`}
                >
                  {viewMode === 'grid' ? '☰' : '⊞'}
                </button>
              </div>
            </header>

            {/* Toolbar */}
            <div className="dashboard-toolbar">
              <div className="selection-info">
                <label>
                  <input
                    type="checkbox"
                    checked={selectedUsers.size > 0 && selectedUsers.size === sortedUsers.length}
                    onChange={handleSelectAll}
                    aria-label="Select all users"
                  />
                  {selectedUsers.size > 0 
                    ? `${selectedUsers.size} of ${sortedUsers.length} selected`
                    : `${sortedUsers.length} users`
                  }
                </label>
              </div>
              <div className="toolbar-actions">
                <button
                  onClick={() => setIsFormOpen(true)}
                  className="btn-primary"
                >
                  Add User
                </button>
                {selectedUsers.size > 0 && (
                  <button
                    onClick={handleBulkDelete}
                    className="btn-danger"
                    disabled={deleteUserMutation.isLoading}
                  >
                    Delete Selected ({selectedUsers.size})
                  </button>
                )}
              </div>
            </div>

            {/* Loading indicator for pending transitions */}
            {isPending && (
              <div className="transition-loading" aria-live="polite">
                Updating results...
              </div>
            )}

            {/* User Grid/List */}
            <main className={`user-content ${viewMode}`}>
              {sortedUsers.length === 0 ? (
                <div className="no-users" role="status">
                  <p>No users found.</p>
                </div>
              ) : (
                <div className="user-grid" role="grid" aria-label="Users list">
                  {sortedUsers.map((user) => (
                    <div key={user.id} className="user-grid-item" role="gridcell">
                      <label>
                        <input
                          type="checkbox"
                          checked={selectedUsers.has(user.id)}
                          onChange={(e) => handleUserSelect(user.id, e.target.checked)}
                          aria-label={`Select ${user.name}`}
                        />
                      </label>
                      <UserCard
                        user={user}
                        onEdit={handleEditUser}
                        onDelete={handleDeleteUser}
                        isSelected={selectedUsers.has(user.id)}
                      />
                    </div>
                  ))}
                </div>
              )}
            </main>

            {/* Lazy loaded sections */}
            <section ref={settingsRef} className="dashboard-analytics">
              {isSettingsVisible && (
                <Suspense fallback={<div>Loading analytics...</div>}>
                  <UserAnalytics users={selectedUsersArray} />
                </Suspense>
              )}
            </section>

            {/* Modal for user form */}
            {isFormOpen && (
              <div 
                className="modal-overlay" 
                onClick={() => setIsFormOpen(false)}
                role="dialog"
                aria-modal="true"
                aria-labelledby="form-title"
              >
                <div 
                  className="modal-content" 
                  onClick={(e) => e.stopPropagation()}
                >
                  <header className="modal-header">
                    <h2 id="form-title">
                      {editingUser ? 'Edit User' : 'Add New User'}
                    </h2>
                    <button 
                      onClick={() => setIsFormOpen(false)}
                      aria-label="Close modal"
                    >
                      ×
                    </button>
                  </header>
                  
                  <form onSubmit={handleSubmit(handleFormSubmit)}>
                    <div className="form-group">
                      <label htmlFor="name">Name</label>
                      <Controller
                        name="name"
                        control={control}
                        render={({ field }) => (
                          <input
                            {...field}
                            id="name"
                            type="text"
                            className={errors.name ? 'error' : ''}
                            aria-describedby={errors.name ? 'name-error' : undefined}
                          />
                        )}
                      />
                      {errors.name && (
                        <span id="name-error" role="alert" className="error-message">
                          {errors.name.message}
                        </span>
                      )}
                    </div>

                    <div className="form-group">
                      <label htmlFor="email">Email</label>
                      <Controller
                        name="email"
                        control={control}
                        render={({ field }) => (
                          <input
                            {...field}
                            id="email"
                            type="email"
                            className={errors.email ? 'error' : ''}
                            aria-describedby={errors.email ? 'email-error' : undefined}
                          />
                        )}
                      />
                      {errors.email && (
                        <span id="email-error" role="alert" className="error-message">
                          {errors.email.message}
                        </span>
                      )}
                    </div>

                    <div className="form-group">
                      <label htmlFor="age">Age</label>
                      <Controller
                        name="age"
                        control={control}
                        render={({ field }) => (
                          <input
                            {...field}
                            id="age"
                            type="number"
                            className={errors.age ? 'error' : ''}
                            aria-describedby={errors.age ? 'age-error' : undefined}
                          />
                        )}
                      />
                      {errors.age && (
                        <span id="age-error" role="alert" className="error-message">
                          {errors.age.message}
                        </span>
                      )}
                    </div>

                    <div className="form-group">
                      <label htmlFor="role">Role</label>
                      <Controller
                        name="role"
                        control={control}
                        render={({ field }) => (
                          <select
                            {...field}
                            id="role"
                            className={errors.role ? 'error' : ''}
                            aria-describedby={errors.role ? 'role-error' : undefined}
                          >
                            <option value="user">User</option>
                            <option value="moderator">Moderator</option>
                            <option value="admin">Admin</option>
                          </select>
                        )}
                      />
                      {errors.role && (
                        <span id="role-error" role="alert" className="error-message">
                          {errors.role.message}
                        </span>
                      )}
                    </div>

                    <div className="form-actions">
                      <button 
                        type="button" 
                        onClick={() => setIsFormOpen(false)}
                        disabled={isSubmitting}
                      >
                        Cancel
                      </button>
                      <button 
                        type="submit" 
                        className="btn-primary"
                        disabled={isSubmitting}
                      >
                        {isSubmitting ? 'Saving...' : (editingUser ? 'Update' : 'Create')}
                      </button>
                    </div>
                  </form>
                </div>
              </div>
            )}
          </div>
        </NotificationContext.Provider>
      </ThemeContext.Provider>
    </UserContext.Provider>
  );
};

AdvancedUserDashboard.propTypes = {
  initialUsers: PropTypes.array,
  onUserAction: PropTypes.func
};

AdvancedUserDashboard.defaultProps = {
  initialUsers: [],
  onUserAction: () => {}
};

export default AdvancedUserDashboard; 