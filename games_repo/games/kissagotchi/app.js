/* global anime */

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
        this.gameInterval = setInterval(() => this.gameLoop(), 5000);
        
        // Save state before closing
        window.addEventListener('beforeunload', () => this.saveState());
        
        // Initial greeting
        this.showMessage("Miau!");
    }

    destroy() {
        if (this.gameInterval) {
            clearInterval(this.gameInterval);
        }
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
            let diffMinutes = Math.floor(diffMs / 60000);
            
            if (diffMinutes > 0) {
                if (this.state.isSleeping) {
                    const energyNeeded = this.MAX_STAT - this.state.energy;
                    const minutesToWakeUp = Math.ceil(energyNeeded / 2);
                    
                    if (diffMinutes >= minutesToWakeUp) {
                        // Kissa herää offline-aikana
                        this.state.energy = this.MAX_STAT;
                        this.state.satiety -= Math.floor(minutesToWakeUp / 2);
                        this.state.isSleeping = false;
                        diffMinutes -= minutesToWakeUp;
                    } else {
                        // Kissa ei ehdi herätä
                        this.state.energy += diffMinutes * 2;
                        this.state.satiety -= Math.floor(diffMinutes / 2);
                        diffMinutes = 0;
                    }
                }
                
                // Jos kissa on hereillä (tai heräsi unesta kesken kaiken)
                if (diffMinutes > 0 && !this.state.isSleeping) {
                    const decrease = Math.floor(diffMinutes / 2);
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
        
        this.triggerAnimation('eat');
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

        this.triggerAnimation('play');
        this.showMessage("Purrrr! 🧶");
        this.saveState();
    }

    toggleSleep() {
        this.state.isSleeping = !this.state.isSleeping;
        this.updateFace();
        
        if (this.state.isSleeping) {
            this.showMessage("Hyvää yötä! 💤");
            this.triggerAnimation('sleep');
        } else {
            this.showMessage("Huomenta! ☀️");
            if (typeof anime !== 'undefined') {
                anime({ targets: this.catContainer, scale: [0.9, 1], duration: 800, easing: 'easeOutElastic(1, .5)' });
            }
        }
        this.saveState();
    }

    createParticles(type) {
        if (typeof anime === 'undefined') return;
        const container = document.querySelector('.pet-display');
        if (!container) return;

        let emojis = [];
        let count = 10;
        let scaleRange = [0.5, 1.5];
        
        if (type === 'eat') {
            emojis = ['🐟', '💖', '✨'];
        } else if (type === 'play') {
            emojis = ['🧶', '⭐', '🎵'];
        } else if (type === 'sleep') {
            emojis = ['Z', 'z', '💤'];
            count = 5;
            scaleRange = [0.8, 1.5];
        } else if (type === 'sad') {
            emojis = ['💧', '😢', '💔'];
            count = 6;
        }

        for (let i = 0; i < count; i++) {
            const p = document.createElement('div');
            p.textContent = emojis[Math.floor(Math.random() * emojis.length)];
            p.style.position = 'absolute';
            p.style.fontSize = type === 'sleep' ? '30px' : '24px';
            p.style.pointerEvents = 'none';
            p.style.zIndex = '100';
            p.style.left = '50%';
            p.style.top = '50%';
            p.style.transform = 'translate(-50%, -50%)';
            p.style.color = type === 'sleep' ? '#A29BFE' : '#FFF';
            p.style.textShadow = '0 2px 8px rgba(0,0,0,0.4)';
            container.appendChild(p);

            const angle = Math.random() * Math.PI * 2;
            const distance = 80 + Math.random() * 100;

            if (type === 'sleep') {
                anime({
                    targets: p,
                    translateX: [-20 + Math.random()*40, -40 + Math.random()*80],
                    translateY: [0, -100 - Math.random()*60],
                    opacity: [0, 1, 0],
                    scale: scaleRange,
                    duration: 3000 + Math.random() * 2000,
                    easing: 'easeOutSine',
                    complete: () => p.remove()
                });
            } else {
                anime({
                    targets: p,
                    translateX: Math.cos(angle) * distance,
                    translateY: Math.sin(angle) * distance - 20,
                    opacity: [1, 0],
                    scale: scaleRange,
                    rotate: Math.random() * 360 - 180,
                    duration: 1000 + Math.random() * 1000,
                    easing: 'easeOutExpo',
                    complete: () => p.remove()
                });
            }
        }
    }

    triggerAnimation(type) {
        if (!this.catContainer) return;
        
        if (this.animTimeout) {
            clearTimeout(this.animTimeout);
        }

        this.catContainer.classList.remove('anim-sleep', 'anim-sad', 'anim-eat', 'anim-play');
        
        if (typeof anime !== 'undefined') {
            anime.remove(this.catContainer);
            this.createParticles(type);
            
            if (type === 'eat') {
                anime({
                    targets: this.catContainer,
                    scale: [1, 1.1, 1],
                    translateY: [0, 5, 0],
                    duration: 400,
                    easing: 'easeInOutQuad',
                    direction: 'alternate',
                    loop: 2
                });
            } else if (type === 'play') {
                anime({
                    targets: this.catContainer,
                    rotate: [0, -15, 15, -15, 0],
                    translateY: [0, -25, -25, -25, 0],
                    duration: 800,
                    easing: 'easeOutElastic(1, .5)'
                });
            }
        } else {
            // Fallback
            this.catContainer.classList.add(`anim-${type}`);
        }
        
        this.animTimeout = setTimeout(() => {
            if (!this.state.isSleeping) {
                if (typeof anime !== 'undefined') {
                    anime({ targets: this.catContainer, rotate: 0, translateY: 0, scale: 1, duration: 400 });
                } else {
                    this.catContainer.classList.remove(`anim-${type}`);
                }
                this.updateFace(); 
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
