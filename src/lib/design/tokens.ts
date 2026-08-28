export const designTokens = {
  palette: {
    day: {
      canvas: '#F4E9CF',
      surface: '#FFF9E8',
      ink: '#182327',
      mutedInk: '#526064',
      safety: '#B83A1B',
      route: '#1F4B57',
      success: '#2E6A4F',
      warning: '#8A5A00',
      danger: '#A52A2A',
      focus: '#006C82',
    },
    night: {
      canvas: '#11191C',
      surface: '#1B272A',
      ink: '#F8EFD9',
      mutedInk: '#BEC7C3',
      safety: '#FF724C',
      route: '#78B9C7',
      success: '#70C49A',
      warning: '#F3B84B',
      danger: '#FF8074',
      focus: '#78B9C7',
    },
  },
  spacePx: [4, 8, 12, 16, 24, 32, 48, 64, 96],
  radiusPx: [0, 2, 6],
  motionMs: [120, 180, 260],
  minimumTargetPx: 44,
} as const;

export type DesignTokens = typeof designTokens;
