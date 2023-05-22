#
# Snowman
#
# ------------------------------------------------------------------------------
# Configuration
# ------------------------------------------------------------------------------

SPACESHIP_SNOWMAN_SHOW="${SPACESHIP_SNOWMAN_SHOW=true}"
SPACESHIP_SNOWMAN_PREFIX="${SPACESHIP_SNOWMAN_PREFIX=""}"
SPACESHIP_SNOWMAN_SUFFIX="${SPACESHIP_SNOWMAN_SUFFIX="$SPACESHIP_PROMPT_DEFAULT_SUFFIX"}"
SPACESHIP_SNOWMAN_SYMBOL="${SPACESHIP_SNOWMAN_SYMBOL="⛄ "}"
SPACESHIP_SNOWMAN_COLOR="${SPACESHIP_SNOWMAN_COLOR="white"}"

# ------------------------------------------------------------------------------
# Section
# ------------------------------------------------------------------------------

spaceship_snowman() {
  [[ $SPACESHIP_SNOWMAN_SHOW == false ]] && return
  [[ -z $_SNOWMAN_ENVIRONMENT_NAME ]] && return

  spaceship::section \
    "$SPACESHIP_SNOWMAN_COLOR" \
    "$SPACESHIP_SNOWMAN_PREFIX" \
    "${SPACESHIP_SNOWMAN_SYMBOL}${_SNOWMAN_ENVIRONMENT_NAME}" \
    "$SPACESHIP_SNOWMAN_SUFFIX"
}
