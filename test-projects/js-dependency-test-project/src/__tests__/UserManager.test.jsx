/**
 * Test file for UserManager component
 * This file should be excluded by smart dependency filtering
 * as test files are typically not needed for code intelligence
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Provider } from 'react-redux';
import { createStore } from 'redux';
import '@testing-library/jest-dom';

import UserManager from '../components/UserManager.jsx';

// Mock store
const mockStore = createStore((state = {
  auth: {
    currentUser: { id: 1, name: 'Test User' },
    permissions: ['user:read', 'user:create', 'user:edit', 'user:delete'],
  },
  ui: { theme: 'default' },
}) => state);

// Mock UserService
jest.mock('../services/UserService', () => ({
  UserService: jest.fn().mockImplementation(() => ({
    initialize: jest.fn().mockResolvedValue(true),
    getUsers: jest.fn().mockResolvedValue({
      data: [
        { id: 1, name: 'John Doe', email: 'john@example.com' },
        { id: 2, name: 'Jane Smith', email: 'jane@example.com' },
      ],
    }),
    searchUsers: jest.fn().mockResolvedValue({ data: [] }),
    createUser: jest.fn().mockResolvedValue({ id: 3, name: 'New User' }),
    updateUser: jest.fn().mockResolvedValue({ id: 1, name: 'Updated User' }),
    deleteUser: jest.fn().mockResolvedValue(true),
    setApiEndpoints: jest.fn(),
  })),
}));

// Test wrapper component
const TestWrapper = ({ children }) => (
  <Provider store={mockStore}>{children}</Provider>
);

describe('UserManager Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('renders UserManager component', () => {
    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    expect(screen.getByText('User Management')).toBeInTheDocument();
  });

  test('displays search input', () => {
    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    const searchInput = screen.getByPlaceholderText('Search users...');
    expect(searchInput).toBeInTheDocument();
  });

  test('handles search input change', async () => {
    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    const searchInput = screen.getByPlaceholderText('Search users...');
    fireEvent.change(searchInput, { target: { value: 'John' } });

    expect(searchInput.value).toBe('John');
  });

  test('displays refresh button', () => {
    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    const refreshButton = screen.getByText('Refresh');
    expect(refreshButton).toBeInTheDocument();
  });

  test('displays add user button when user has create permission', () => {
    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    const addButton = screen.getByText('Add User');
    expect(addButton).toBeInTheDocument();
  });

  test('handles refresh button click', async () => {
    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    const refreshButton = screen.getByText('Refresh');
    fireEvent.click(refreshButton);

    // Test that refresh was triggered
    await waitFor(() => {
      expect(refreshButton).not.toBeDisabled();
    });
  });

  test('toggles form visibility when add user button is clicked', async () => {
    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    const addButton = screen.getByText('Add User');
    fireEvent.click(addButton);

    // Check if button text changed to Cancel
    expect(screen.getByText('Cancel')).toBeInTheDocument();
  });

  test('handles error states properly', async () => {
    // Mock error response
    const mockUserService = require('../services/UserService').UserService;
    mockUserService.mockImplementation(() => ({
      initialize: jest.fn().mockRejectedValue(new Error('Network error')),
      setApiEndpoints: jest.fn(),
    }));

    render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    // Wait for error to be displayed
    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeInTheDocument();
    });
  });

  test('applies correct CSS classes based on theme', () => {
    const { container } = render(
      <TestWrapper>
        <UserManager theme="dark" />
      </TestWrapper>
    );

    const userManagerDiv = container.querySelector('.user-manager');
    expect(userManagerDiv).toHaveClass('theme-dark');
  });

  test('handles component unmounting gracefully', () => {
    const { unmount } = render(
      <TestWrapper>
        <UserManager />
      </TestWrapper>
    );

    // Should not throw errors on unmount
    expect(() => unmount()).not.toThrow();
  });
});

// Integration test
describe('UserManager Integration', () => {
  test('integrates with Redux store correctly', () => {
    const testStore = createStore((state = {
      auth: { currentUser: null, permissions: [] },
      ui: { theme: 'light' },
    }) => state);

    render(
      <Provider store={testStore}>
        <UserManager />
      </Provider>
    );

    // Should render without crashing even with minimal state
    expect(screen.getByText('User Management')).toBeInTheDocument();
  });
}); 