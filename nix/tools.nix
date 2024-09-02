{ buildNpmPackage, fetchFromGitHub, symlinkJoin, writeShellScriptBin, writeText
, lib, pkgs }:
let inherit (lib) getExe;
in {

  formatter = let
    inherit (pkgs.nodePackages) prettier;

    jinja-prettier = buildNpmPackage rec {
      pname = "prettier-plugin-jinja-template";
      version = "1.5.0";

      src = fetchFromGitHub {
        owner = "davidodenwald";
        repo = pname;
        rev = "160e05cadd97ceb84fd891d66a49da30f43e3519";
        hash = "sha256-RzUMam3XhE+idnEgQf+p7L1nHRSFGbwvzasW/yL8rPw=";
      };

      npmDepsHash = "sha256-vIzm2tGY6s3+SgbwGLwU0w8ZtwID5Set/tRy9/dhPKQ=";
    };
    prettierConfig = writeText ".prettierrc" # yaml
      ''
        tabWidth: 2 
        plugins:
          - prettier-plugin-jinja-template
        overrides:
        - files:
          - "*.html"
          options:
            parser: jinja-template
      '';

  in symlinkJoin {
    name = "prettier";
    paths = [ prettier jinja-prettier ];
    buildInputs = [ pkgs.makeWrapper ];
    postBuild = ''
      wrapProgram $out/bin/prettier \
        --add-flags "--config ${prettierConfig}"
    '';
  };

  buildTailwind = let
    tailwindConfig = writeText "tailwind.config.js" # javascript
      ''
        module.exports = {
          content: ["./templates/**/*.html"],
          theme: {
            extend: {
              colors: {
                base: "#232136",
                surface: "#2a273f",
                overlay: "#393552",
                muted: "#6e6a86",
                subtle: "#908caa",
                text: "#e0def4",
                love: "#eb6f92",
                gold: "#f6c177",
                rose: "#ea9a97",
                pine: "#3e8fb0",
                foam: "#9ccfd8",
                iris: "#c4a7e7",
                highlightLow: "#2a283e",
                highlightMed: "#44415a",
                highlightHigh: "#56526e",
              },
            },
          },
          plugins: [require("@tailwindcss/container-queries")],
        };
      '';

    postcssConfig = writeText "postcss.config.cjs" # javascript
      ''
        module.exports = {
          plugins: {
            tailwindcss: {},
            autoprefixer: {},
          },
        };
      '';

  in writeShellScriptBin "build-styles" # sh
  ''
    ${getExe pkgs.tailwindcss} --minify  \
              --config ${tailwindConfig}  \
              --postcss ${postcssConfig}  \
              --input ./static/styles.css \
              --output ./static/_dist.css
  '';
}
