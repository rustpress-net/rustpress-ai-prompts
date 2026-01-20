/**
 * Business Starter Theme - Navigation
 */

(function() {
    'use strict';

    const Navigation = {
        init() {
            this.initMobileMenu();
            this.initDropdowns();
            this.initKeyboardNav();
        },

        // Mobile menu toggle
        initMobileMenu() {
            const toggle = document.querySelector('.menu-toggle');
            const mobileNav = document.querySelector('.mobile-navigation');

            if (!toggle || !mobileNav) return;

            toggle.addEventListener('click', () => {
                const isOpen = mobileNav.hidden === false;

                mobileNav.hidden = isOpen;
                toggle.setAttribute('aria-expanded', !isOpen);
                toggle.classList.toggle('is-active', !isOpen);

                // Prevent body scroll when menu is open
                document.body.classList.toggle('menu-open', !isOpen);
            });

            // Close on resize to desktop
            window.addEventListener('resize', () => {
                if (window.innerWidth >= 1024) {
                    mobileNav.hidden = true;
                    toggle.setAttribute('aria-expanded', 'false');
                    toggle.classList.remove('is-active');
                    document.body.classList.remove('menu-open');
                }
            });
        },

        // Dropdown menus
        initDropdowns() {
            const menuItems = document.querySelectorAll('.nav-menu > li');

            menuItems.forEach(item => {
                const submenu = item.querySelector('.sub-menu');
                if (!submenu) return;

                const link = item.querySelector('a');

                // Add dropdown indicator
                const indicator = document.createElement('span');
                indicator.className = 'dropdown-indicator';
                indicator.innerHTML = `
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                `;
                link.appendChild(indicator);

                // Mouse events
                item.addEventListener('mouseenter', () => {
                    this.openDropdown(item, submenu);
                });

                item.addEventListener('mouseleave', () => {
                    this.closeDropdown(item, submenu);
                });

                // Touch/click for mobile
                link.addEventListener('click', (e) => {
                    if (window.innerWidth < 1024 && submenu) {
                        e.preventDefault();
                        const isOpen = item.classList.contains('dropdown-open');
                        this.closeAllDropdowns();
                        if (!isOpen) {
                            this.openDropdown(item, submenu);
                        }
                    }
                });
            });

            // Close dropdowns when clicking outside
            document.addEventListener('click', (e) => {
                if (!e.target.closest('.nav-menu')) {
                    this.closeAllDropdowns();
                }
            });
        },

        openDropdown(item, submenu) {
            item.classList.add('dropdown-open');
            submenu.setAttribute('aria-hidden', 'false');
        },

        closeDropdown(item, submenu) {
            item.classList.remove('dropdown-open');
            submenu.setAttribute('aria-hidden', 'true');
        },

        closeAllDropdowns() {
            document.querySelectorAll('.dropdown-open').forEach(item => {
                item.classList.remove('dropdown-open');
                const submenu = item.querySelector('.sub-menu');
                if (submenu) {
                    submenu.setAttribute('aria-hidden', 'true');
                }
            });
        },

        // Keyboard navigation
        initKeyboardNav() {
            const navMenu = document.querySelector('.nav-menu');
            if (!navMenu) return;

            navMenu.addEventListener('keydown', (e) => {
                const currentItem = document.activeElement.closest('li');
                if (!currentItem) return;

                const items = Array.from(navMenu.querySelectorAll(':scope > li > a'));
                const currentIndex = items.indexOf(document.activeElement);

                switch (e.key) {
                    case 'ArrowRight':
                        e.preventDefault();
                        const nextIndex = (currentIndex + 1) % items.length;
                        items[nextIndex].focus();
                        break;

                    case 'ArrowLeft':
                        e.preventDefault();
                        const prevIndex = (currentIndex - 1 + items.length) % items.length;
                        items[prevIndex].focus();
                        break;

                    case 'ArrowDown':
                        const submenu = currentItem.querySelector('.sub-menu');
                        if (submenu) {
                            e.preventDefault();
                            this.openDropdown(currentItem, submenu);
                            const firstSubItem = submenu.querySelector('a');
                            if (firstSubItem) firstSubItem.focus();
                        }
                        break;

                    case 'Escape':
                        this.closeAllDropdowns();
                        document.activeElement.blur();
                        break;
                }
            });
        }
    };

    // Initialize
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => Navigation.init());
    } else {
        Navigation.init();
    }

    window.BusinessStarterNavigation = Navigation;
})();
