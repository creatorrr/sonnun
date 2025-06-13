// AIDEV-NOTE: Jest setup file for test environment configuration

// Mock Tauri APIs for testing
const mockInvoke = jest.fn()
jest.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

// Mock browser APIs that may not be available in test environment
if (typeof window !== 'undefined') {
  Object.defineProperty(window, 'crypto', {
    value: {
      subtle: {
        digest: jest.fn().mockResolvedValue(new ArrayBuffer(32))
      }
    }
  })
}

// Reset mocks before each test
beforeEach(() => {
  mockInvoke.mockReset()
})