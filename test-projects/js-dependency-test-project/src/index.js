/**
 * Main entry point for the JavaScript dependency test project
 * This demonstrates the UserManager component and dependency patterns
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import { Provider } from 'react-redux';
import { createStore } from 'redux';

import UserManager from './components/UserManager.jsx';
import { Logger } from './utils/Logger';
import { Config } from './config/Config';

// Initialize logger
const logger = new Logger('App');

// Simple Redux store for testing
const initialState = {
  auth: {
    currentUser: null,
    permissions: ['user:read', 'user:create', 'user:edit', 'user:delete'],
  },
  ui: {
    theme: 'default',
  },
};

const rootReducer = (state = initialState, action) => {
  switch (action.type) {
    case 'USER_CHANGED':
      return {
        ...state,
        auth: { ...state.auth, currentUser: action.payload },
      };
    default:
      return state;
  }
};

const store = createStore(rootReducer);

// App component
const App = () => {
  logger.info('App component rendering');
  
  return (
    <Provider store={store}>
      <div className="app">
        <header className="app-header">
          <h1>JavaScript Dependency Test Project</h1>
          <p>Testing GCore MCP dependency scanning with React components</p>
        </header>
        
        <main className="app-main">
          <UserManager />
        </main>
        
        <footer className="app-footer">
          <p>© 2024 GCore Test Suite</p>
        </footer>
      </div>
    </Provider>
  );
};

// Initialize application
const initializeApp = async () => {
  try {
    logger.info('Initializing JavaScript dependency test application');
    
    // Initialize configuration
    const config = Config.getInstance();
    await config.initialize({
      api: {
        baseUrl: 'http://localhost:3001',
        timeout: 10000,
        retries: 3,
      },
      logging: {
        level: 'INFO',
      },
    });
    
    // Render app
    const root = ReactDOM.createRoot(document.getElementById('root'));
    root.render(<App />);
    
    logger.info('Application initialized successfully');
    
  } catch (error) {
    logger.error('Failed to initialize application', error);
  }
};

// Start the application
if (typeof window !== 'undefined') {
  // Browser environment
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
  } else {
    initializeApp();
  }
} else {
  // Node.js environment (for testing)
  logger.info('Running in Node.js environment');
}

export default App; 