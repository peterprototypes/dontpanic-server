import { describe, it } from 'vitest';
import { render } from '@testing-library/react';

import App from './App';

describe('something truthy and falsy', () => {
  it('renders the Login component', () => {
    render(<App />);
  });
});