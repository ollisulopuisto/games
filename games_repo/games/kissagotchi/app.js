class Kissagotchi {
    constructor() {
        this.MAX_STAT = 100;
        
        // Default state
        this.state = {
            satiety: 100,
            happiness: 100,
            energy: 100,
            lastUpdate: Date.now(),
            isSleeping: false
        };

        // DOM elements
        this.bars = {
            satiety: document.getElementById('hunger-bar'),
            happiness: document.getElementById('happiness-bar'),
            energy: document.getElementById('energy-bar')
        };
        
        this.catContainer = document.getElementById('cat');
        this.statusMsg = document.getElementById('status-message');
        this.leftEye = document.querySelector('.left-eye');
        this.rightEye = document.querySelector('.right-eye');
        this.mouth = document.querySelector('path[stroke="#4A4A4A"]');

        this.animTimeout = null;

        if (this.bars.satiety) this.init();
    }

    init() {
        this.loadState();
        this.updateBars();
        this.updateFace();
        
        // Event listeners
        document.getElementById('btn-feed')?.addEventListener('click', () => this.feed());
        document.getElementById('btn-play')?.addEventListener('click', () => this.play());
        document.getElementById('btn-sleep')?.addEventListener('click', () => this.toggleSleep());

        // Game loop (updates every 5 seconds)
        setInterval(() => this.gameLoop(), 5000);
        
        // Save state before closing
        window.addEventListener('beforeunload', () => this.saveState());
        
        // Initial greeting
        this.showMessage("Miau!");
    }

    loadState() {
        try {
            const saved = localStorage.getItem('kissagotchi_state');
            if (saved) {
                const parsed = JSON.parse(saved);
                this.state = { ...this.state, ...parsed };
                // handle legacy state
                if (this.state.hunger !== undefined) {
                    this.state.satiety = this.state.hunger;
                    delete this.state.hunger;
                }
                this.calculateOfflineProgress();
            }
        } catch (e) {
            console.warn("Failed to load state", e);
        }
    }

    saveState() {
        try {
            this.state.lastUpdate = Date.now();
            localStorage.setItem('kissagotchi_state', JSON.stringify(this.state));
        } catch (e) {
            console.warn("Failed to save state", e);
        }
    }

    calculateOfflineProgress() {
        const now = Date.now();
        const diffMs = now - this.state.lastUpdate;
        
        // Prevent exploit if time goes backwards or is too far in future (e.g. > 1 week)
        if (diffMs > 0 && diffMs < 7 * 24 * 60 * 60 * 1000) {
            const diffMinutes = Math.floor(diffMs / 60000);
            
            if (diffMinutes > 0) {
                const decrease = Math.floor(diffMinutes / 2);
                
                if (this.state.isSleeping) {
                    this.state.energy = Math.min(this.MAX_STAT, this.state.energy + (diffMinutes * 2));
                    this.state.satiety -= decrease;
                } else {
                    this.state.satiety -= decrease;
                    this.state.happiness -= decrease;
                    this.state.energy -= decrease;
                }
                
                this.clampStats();
            }
        } else if (diffMs < 0) {
            this.state.lastUpdate = now; // Time went backwards, reset last update
        }
    }

    gameLoop() {
        if (this.state.isSleeping) {
            this.state.energy += 2;
            this.state.satiety -= 0.5;
            
            // Wake up if fully rested
            if (this.state.energy >= this.MAX_STAT) {
                this.toggleSleep();
                this.showMessage("Olen virkeä!");
            }
        } else {
            this.state.satiety -= 0.5;
            this.state.happiness -= 0.5;
            this.state.energy -= 0.3;
        }

        this.clampStats();
        this.updateBars();
        this.updateFace();
        this.saveState();
    }

    clampStats() {
        this.state.satiety = Math.max(0, Math.min(this.MAX_STAT, this.state.satiety));
        this.state.happiness = Math.max(0, Math.min(this.MAX_STAT, this.state.happiness));
        this.state.energy = Math.max(0, Math.min(this.MAX_STAT, this.state.energy));
    }

    updateBars() {
        if (!this.bars.satiety) return;
        this.bars.satiety.style.width = `${this.state.satiety}%`;
        this.bars.happiness.style.width = `${this.state.happiness}%`;
        this.bars.energy.style.width = `${this.state.energy}%`;
    }

    updateFace() {
        if (!this.catContainer) return;
        
        // Remove old classes
        this.catContainer.classList.remove('anim-sleep', 'anim-sad');
        
        // Change expression based on stats
        if (this.state.isSleeping) {
            // Closed eyes
            this.leftEye.setAttribute('r', '2');
            this.rightEye.setAttribute('r', '2');
            this.mouth.setAttribute('d', 'M 95 135 L 105 135'); // Neutral mouth
            this.catContainer.classList.add('anim-sleep');
        } else if (this.state.satiety < 30 || this.state.happiness < 30) {
            // Sad face
            this.leftEye.setAttribute('r', '8');
            this.rightEye.setAttribute('r', '8');
            this.mouth.setAttribute('d', 'M 90 140 Q 100 130 110 140'); // Frown
            this.catContainer.classList.add('anim-sad');
        } else {
            // Happy face
            this.leftEye.setAttribute('r', '8');
            this.rightEye.setAttribute('r', '8');
            this.mouth.setAttribute('d', 'M 90 135 Q 95 142 100 135 Q 105 142 110 135'); // Smile
        }
    }

    feed() {
        if (this.state.isSleeping) return this.showMessage("Zzz... en voi syödä nukkuessa.");
        
        this.state.satiety += 20;
        this.state.energy += 5;
        this.clampStats();
        this.updateBars();
        this.updateFace();
        
        this.triggerAnimation('anim-eat');
        this.showMessage("Nam nam! 🐟");
        this.saveState();
    }

    play() {
        if (this.state.isSleeping) return this.showMessage("Zzz... haluan nukkua.");
        if (this.state.energy < 20) return this.showMessage("Olen liian väsynyt leikkimään...");
        if (this.state.satiety < 20) return this.showMessage("Olen liian nälkäinen...");

        this.state.happiness += 20;
        this.state.energy -= 15;
        this.state.satiety -= 10;
        this.clampStats();
        this.updateBars();
        this.updateFace();

        this.triggerAnimation('anim-play');
        this.showMessage("Purrrr! 🧶");
        this.saveState();
    }

    toggleSleep() {
        this.state.isSleeping = !this.state.isSleeping;
        this.updateFace();
        
        if (this.state.isSleeping) {
            this.showMessage("Hyvää yötä! 💤");
        } else {
            this.showMessage("Huomenta! ☀️");
        }
        this.saveState();
    }

    triggerAnimation(className) {
        if (!this.catContainer) return;
        
        if (this.animTimeout) {
            clearTimeout(this.animTimeout);
        }

        this.catContainer.classList.remove('anim-eat', 'anim-play', 'anim-sad');
        // Force reflow
        void this.catContainer.offsetWidth;
        this.catContainer.classList.add(className);
        
        this.animTimeout = setTimeout(() => {
            if (!this.state.isSleeping) {
                this.catContainer.classList.remove(className);
                this.updateFace(); // restores default idle class
            }
        }, 1500);
    }

    showMessage(text) {
        if (!this.statusMsg) return;
        this.statusMsg.textContent = text;
        this.statusMsg.classList.remove('show');
        void this.statusMsg.offsetWidth; // Force reflow
        this.statusMsg.classList.add('show');
    }
}

// Export for testing, or start app when DOM loads
if (typeof module !== 'undefined' && module.exports) {
    module.exports = Kissagotchi;
} else {
    document.addEventListener('DOMContentLoaded', () => {
        window.game = new Kissagotchi();
    });
}
