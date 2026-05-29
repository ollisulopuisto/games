const fs = require('fs');
const path = require('path');

// Load HTML template
const html = fs.readFileSync(path.resolve(__dirname, './index.html'), 'utf8');

describe('Kissagotchi TDD', () => {
    let Kissagotchi;
    let games = [];

    const createGame = () => {
        const game = new Kissagotchi();
        games.push(game);
        return game;
    };

    beforeEach(() => {
        // Setup document body
        document.documentElement.innerHTML = html.toString();
        // Clear local storage
        localStorage.clear();
        Kissagotchi = require('./app.js');
    });

    afterEach(() => {
        games.forEach(g => g.destroy());
        games = [];
    });

    it('should initialize with max stats', () => {
        const game = createGame();
        expect(game.state.satiety).toBe(100);
        expect(game.state.happiness).toBe(100);
        expect(game.state.energy).toBe(100);
    });

    it('should decrease stats over time', () => {
        const game = createGame();
        game.gameLoop();
        expect(game.state.satiety).toBeLessThan(100);
        expect(game.state.happiness).toBeLessThan(100);
        expect(game.state.energy).toBeLessThan(100);
    });

    it('should increase satiety when feeding', () => {
        const game = createGame();
        game.state.satiety = 50; // Artificially lower it
        game.feed();
        expect(game.state.satiety).toBe(70);
        expect(game.state.energy).toBe(100); // capped at 100
    });

    it('should increase happiness and decrease energy when playing', () => {
        const game = createGame();
        game.state.happiness = 50;
        game.play();
        expect(game.state.happiness).toBe(70);
        expect(game.state.energy).toBe(85); // 100 - 15
        expect(game.state.satiety).toBe(90); // 100 - 10
    });

    it('should not allow playing when too tired', () => {
        const game = createGame();
        game.state.energy = 10;
        game.play();
        expect(game.state.happiness).toBe(100); // Should not increase
    });

    it('should restore energy when sleeping', () => {
        const game = createGame();
        game.state.energy = 50;
        game.toggleSleep();
        game.gameLoop();
        expect(game.state.energy).toBe(52); // Sleeps gives +2
        expect(game.state.isSleeping).toBe(true);
    });
});
