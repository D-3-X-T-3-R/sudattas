import { render, screen } from '@testing-library/react';
import App from './App';

test('renders brand and shop section', () => {
  render(<App />);
  expect(screen.getByText(/SUDATTA'S/i)).toBeInTheDocument();
  expect(screen.getByText(/SHOP/i)).toBeInTheDocument();
});
