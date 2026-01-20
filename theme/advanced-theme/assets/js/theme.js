/**
 * Business Starter Theme - Main JavaScript
 */

(function() {
    'use strict';

    // Theme state
    const Theme = {
        // Initialize theme
        init() {
            this.initThemeToggle();
            this.initBackToTop();
            this.initStickyHeader();
            this.initSearchToggle();
        },

        // Theme toggle (dark/light mode)
        initThemeToggle() {
            const toggle = document.querySelector('.theme-toggle');
            if (!toggle) return;

            // Get saved theme or default to 'auto'
            const savedTheme = localStorage.getItem('theme') || 'auto';
            this.setTheme(savedTheme);

            toggle.addEventListener('click', () => {
                const current = document.documentElement.dataset.theme;
                const next = current === 'dark' ? 'light' : 'dark';
                this.setTheme(next);
                localStorage.setItem('theme', next);
            });
        },

        setTheme(theme) {
            document.documentElement.dataset.theme = theme;

            // Update ARIA
            const toggle = document.querySelector('.theme-toggle');
            if (toggle) {
                toggle.setAttribute('aria-label',
                    theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'
                );
            }
        },

        // Back to top button
        initBackToTop() {
            const button = document.getElementById('back-to-top');
            if (!button) return;

            // Show/hide based on scroll position
            const toggleVisibility = () => {
                if (window.scrollY > 300) {
                    button.classList.add('visible');
                } else {
                    button.classList.remove('visible');
                }
            };

            window.addEventListener('scroll', toggleVisibility, { passive: true });
            toggleVisibility();

            // Scroll to top on click
            button.addEventListener('click', () => {
                window.scrollTo({
                    top: 0,
                    behavior: 'smooth'
                });
            });
        },

        // Sticky header with shrink effect
        initStickyHeader() {
            const header = document.querySelector('.site-header[data-sticky="true"]');
            if (!header) return;

            let lastScroll = 0;

            window.addEventListener('scroll', () => {
                const currentScroll = window.scrollY;

                // Add/remove scrolled class
                if (currentScroll > 50) {
                    header.classList.add('is-scrolled');
                } else {
                    header.classList.remove('is-scrolled');
                }

                // Hide on scroll down, show on scroll up
                if (currentScroll > lastScroll && currentScroll > 100) {
                    header.classList.add('is-hidden');
                } else {
                    header.classList.remove('is-hidden');
                }

                lastScroll = currentScroll;
            }, { passive: true });
        },

        // Search toggle
        initSearchToggle() {
            const toggle = document.querySelector('.search-toggle');
            const searchPanel = document.querySelector('.header-search');
            const closeBtn = document.querySelector('.search-close');
            const searchInput = document.querySelector('.header-search .search-field');

            if (!toggle || !searchPanel) return;

            toggle.addEventListener('click', () => {
                searchPanel.hidden = !searchPanel.hidden;
                toggle.setAttribute('aria-expanded', !searchPanel.hidden);

                if (!searchPanel.hidden && searchInput) {
                    searchInput.focus();
                }
            });

            if (closeBtn) {
                closeBtn.addEventListener('click', () => {
                    searchPanel.hidden = true;
                    toggle.setAttribute('aria-expanded', 'false');
                    toggle.focus();
                });
            }

            // Close on Escape
            document.addEventListener('keydown', (e) => {
                if (e.key === 'Escape' && !searchPanel.hidden) {
                    searchPanel.hidden = true;
                    toggle.setAttribute('aria-expanded', 'false');
                    toggle.focus();
                }
            });
        }
    };

    // Initialize on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => Theme.init());
    } else {
        Theme.init();
    }

    // Export for external use
    window.BusinessStarterTheme = Theme;
})();
