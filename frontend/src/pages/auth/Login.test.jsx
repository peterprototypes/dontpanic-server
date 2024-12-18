import { describe, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { createRoutesStub } from "react-router";

import Login from './Login';

describe('something truthy and falsy', () => {
  it('renders the Login component', () => {
    const Stub = createRoutesStub([{
      path: "/auth/login",
      Component: Login,
    }]);

    render(<Stub initialEntries={["/auth/login"]} />);

    screen.debug();
  });
});