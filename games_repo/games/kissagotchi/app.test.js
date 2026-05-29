const fs = require('fs');
const path = require('path');

// Load HTML template
const html = fs.readFileSync(path.resolve(__dirname, './index.html'), 'utf8');

describe('Kissagotchi TDD', () => {
    let Kissagotchi;

    beforeEach(() => {
        // Setup document body
        document.documentElement.innerHTML = html.toString();
        // Clear local storage
        localStorage.clear();
        // Load the class
        Kissagotchi = require('./app.js');
    });

    it('should initialize with max stats', () => {
        const game = new Kissagotchi();
        expect(game.state.hunger).toBe(100);
        expect(game.state.happiness).toBe(100);
        expect(game.state.energy).toBe(100);
    });

    it('should decrease stats over time', () => {
        const game = new Kissagotchi();
        game.gameLoop();
        expect(game.state.hunger).toBeLessThan(100);
        expect(game.state.happiness).toBeLessThan(100);
        expect(game.state.energy).toBeLessThan(100);
    });

    it('should increase hunger when feeding', () => {
        const game = new Kissagotchi();
        game.state.hunger = 50; // Artificially lower it
        game.feed();
        expect(game.state.hunger).toBe(70);
        expect(game.state.energy).toBe(100); // capped at 100
    });

    it('should increase happiness and decrease energy when playing', () => {
        const game = new Kissagotchi();
        game.state.happiness = 50;
        game.play();
        expect(game.state.happiness).toBe(70);
        expect(game.state.energy).toBe(85); // 100 - 15
        expect(game.state.hunger).toBe(90); // 100 - 10
    });

    it('should not allow playing when too tired', () => {
        const game = new Kissagotchi();
        game.state.energy = 10;
        game.play();
        expect(game.state.energy).toBe(10); // Still 10, play was rejected
    });

    it('should restore energy when sleeping', () => {
        const game = new Kissagotchi();
        game.state.energy = 50;
        game.toggleSleep();
        game.gameLoop();
        expect(game.state.energy).toBe(52); // Sleeps gives +2
        expect(game.state.isSleeping).toBe(true);
    });
});
