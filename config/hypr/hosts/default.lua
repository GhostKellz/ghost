-- Fallback profile for machines without hosts/<hostname>.lua: every output
-- uses its preferred mode via the fallback monitor rule.
return {
    monitors = {},
    env = {},
}
