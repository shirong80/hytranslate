import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { SourceLanguageSelect } from './source-language-select';

describe('SourceLanguageSelect', () => {
  it('renders detected badge when Auto + resolvedLanguage is set', () => {
    render(
      <SourceLanguageSelect value="Auto" onChange={vi.fn()} resolvedLanguage="ChineseSimplified" />,
    );
    // badge 가 "자동: 중국어 (간체)" 텍스트로 노출.
    expect(screen.getByRole('status')).toHaveTextContent('자동: 중국어 (간체)');
  });

  it('does not render detected badge when value is not Auto', () => {
    render(<SourceLanguageSelect value="Korean" onChange={vi.fn()} resolvedLanguage="Korean" />);
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('does not render detected badge when resolvedLanguage is null', () => {
    render(<SourceLanguageSelect value="Auto" onChange={vi.fn()} resolvedLanguage={null} />);
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('renders Korean detected label', () => {
    render(<SourceLanguageSelect value="Auto" onChange={vi.fn()} resolvedLanguage="Korean" />);
    expect(screen.getByRole('status')).toHaveTextContent('자동: 한국어');
  });
});
