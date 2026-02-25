//generated part by rust handle_install:
//(function () {
//	const available_themes = { ayu_dark: "Ayu Dark", ... };
//	const default_theme = "ayu_dark";
	const html = document.querySelector('html');
	html.setAttribute('code-theme', get_theme());



	function menu() {
		let menu = `
	<button id="mdbook-code-theme-toggle" class="icon-button" type="button" title="Change code theme" aria-label="Change code theme" aria-haspopup="true" aria-expanded="false" aria-controls="mdbook-code-theme-list">
<span class="fa-svg"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 640"><!--!Font Awesome Free v7.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2026 Fonticons, Inc.--><path d="M392.8 65.2C375.8 60.3 358.1 70.2 353.2 87.2L225.2 535.2C220.3 552.2 230.2 569.9 247.2 574.8C264.2 579.7 281.9 569.8 286.8 552.8L414.8 104.8C419.7 87.8 409.8 70.1 392.8 65.2zM457.4 201.3C444.9 213.8 444.9 234.1 457.4 246.6L530.8 320L457.4 393.4C444.9 405.9 444.9 426.2 457.4 438.7C469.9 451.2 490.2 451.2 502.7 438.7L598.7 342.7C611.2 330.2 611.2 309.9 598.7 297.4L502.7 201.4C490.2 188.9 469.9 188.9 457.4 201.4zM182.7 201.3C170.2 188.8 149.9 188.8 137.4 201.3L41.4 297.3C28.9 309.8 28.9 330.1 41.4 342.6L137.4 438.6C149.9 451.1 170.2 451.1 182.7 438.6C195.2 426.1 195.2 405.8 182.7 393.3L109.3 320L182.6 246.6C195.1 234.1 195.1 213.8 182.6 201.3z"/></svg></span>
	</button>
	<ul id="mdbook-code-theme-list" class="theme-popup" aria-label="Code themes" role="menu" style="display:none;">
	`;

		for (let theme in available_themes) {
			menu += `\t<li role="none"><button role="menuitem" class="theme" id=mdbook-code-theme-${theme}>${available_themes[theme]}</button></li>`
		}
		menu += '</ul>'
		return menu;
	}

	document.querySelector('div#mdbook-menu-bar > .left-buttons').insertAdjacentHTML("beforeend", menu());

	const themeToggleButton = document.getElementById('mdbook-code-theme-toggle');
	const themePopup = document.getElementById('mdbook-code-theme-list');

    themeToggleButton.addEventListener('click', function() {
        if (themePopup.style.display === 'block') {
            hideThemes();
        } else {
            showThemes();
        }
    });

    themePopup.addEventListener('click', function(e) {
        let theme;
        if (e.target.className === 'theme') {
            theme = e.target.id;
        } else if (e.target.parentElement.className === 'theme') {
            theme = e.target.parentElement.id;
        } else {
            return;
        }
        theme = theme.replace(/^mdbook-code-theme-/, '');

		set_theme(theme);
    });

    function hideThemes() {
        themePopup.style.display = 'none';
        themeToggleButton.setAttribute('aria-expanded', false);
        themeToggleButton.focus();
    }

    function showThemes() {
		updateThemeSelected()
        themePopup.style.display = 'block';
        themeToggleButton.setAttribute('aria-expanded', true);
        themePopup.querySelector('button#mdbook-code-theme-' + get_theme()).focus();
    }

    function get_theme() {
        const theme = get_saved_theme();
        if (theme === null || theme === undefined || available_themes[theme] === null) {
			console.log("Get default theme.")
			return default_theme;
        } else {
			console.log("Get theme: ", theme)
            return theme;
        }
    }

    function get_saved_theme() {
		let theme = localStorage.getItem('mdbook-code-theme') ?? default_theme;
		return theme;
    }

    function set_theme(theme, store = true) {
		 if (store) {
            try {
				console.log("save theme: ", theme)
                localStorage.setItem('mdbook-code-theme', theme);
            } catch {
                // ignore error.
            }
        }
		html.setAttribute('code-theme', theme);
		updateThemeSelected();
	}

    function updateThemeSelected() {
        themePopup.querySelectorAll('#mdbook-code-theme-list .theme-selected').forEach(function(el) {
            el.classList.remove('theme-selected');
        });
        const selected = get_theme();
        let element = themePopup.querySelector('button#mdbook-code-theme-' + selected);
        if (element === null) {
            // Fall back in case there is no "Default" item.
            element = themePopup.querySelector('button#mdbook-code-theme-' + get_theme());
        }
        element.classList.add('theme-selected');
    }

    themePopup.addEventListener('focusout', function(e) {
        // e.relatedTarget is null in Safari and Firefox on macOS (see workaround below)
        if (!!e.relatedTarget &&
            !themeToggleButton.contains(e.relatedTarget) &&
            !themePopup.contains(e.relatedTarget)
        ) {
            hideThemes();
        }
    });

   	// Should not be needed, but it works around an issue on macOS & iOS:
    // https://github.com/rust-lang/mdBook/issues/628
    document.addEventListener('click', function(e) {
        if (themePopup.style.display === 'block' &&
            !themeToggleButton.contains(e.target) &&
            !themePopup.contains(e.target)
        ) {
            hideThemes();
        }
    });

})();
