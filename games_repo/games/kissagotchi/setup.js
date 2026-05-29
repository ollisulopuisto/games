global.localStorage = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  clear: vi.fn(),
};

global.anime = vi.fn(() => ({
  add: vi.fn().mockReturnThis()
}));
global.anime.timeline = vi.fn(() => ({
  add: vi.fn().mockReturnThis()
}));
global.anime.stagger = vi.fn();
global.anime.remove = vi.fn();
