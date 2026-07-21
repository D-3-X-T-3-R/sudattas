---
name: Heritage Luxe
colors:
  surface: '#fbf9f8'
  surface-dim: '#dbd9d9'
  surface-bright: '#fbf9f8'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#f5f3f3'
  surface-container: '#efeded'
  surface-container-high: '#eae8e7'
  surface-container-highest: '#e4e2e2'
  on-surface: '#1b1c1c'
  on-surface-variant: '#414845'
  inverse-surface: '#303030'
  inverse-on-surface: '#f2f0f0'
  outline: '#727975'
  outline-variant: '#c1c8c3'
  surface-tint: '#466559'
  primary: '#001b12'
  on-primary: '#ffffff'
  primary-container: '#123026'
  on-primary-container: '#79998b'
  inverse-primary: '#adcebf'
  secondary: '#775a19'
  on-secondary: '#ffffff'
  secondary-container: '#fed488'
  on-secondary-container: '#785a1a'
  tertiary: '#161714'
  on-tertiary: '#ffffff'
  tertiary-container: '#2a2b28'
  on-tertiary-container: '#93928e'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#c9eadb'
  primary-fixed-dim: '#adcebf'
  on-primary-fixed: '#022017'
  on-primary-fixed-variant: '#2f4d41'
  secondary-fixed: '#ffdea5'
  secondary-fixed-dim: '#e9c176'
  on-secondary-fixed: '#261900'
  on-secondary-fixed-variant: '#5d4201'
  tertiary-fixed: '#e4e2dd'
  tertiary-fixed-dim: '#c8c6c2'
  on-tertiary-fixed: '#1b1c19'
  on-tertiary-fixed-variant: '#474744'
  background: '#fbf9f8'
  on-background: '#1b1c1c'
  surface-variant: '#e4e2e2'
typography:
  headline-xl:
    fontFamily: Playfair Display
    fontSize: 48px
    fontWeight: '600'
    lineHeight: '1.2'
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Playfair Display
    fontSize: 32px
    fontWeight: '500'
    lineHeight: '1.3'
  headline-md:
    fontFamily: Playfair Display
    fontSize: 24px
    fontWeight: '500'
    lineHeight: '1.4'
  body-lg:
    fontFamily: Plus Jakarta Sans
    fontSize: 18px
    fontWeight: '400'
    lineHeight: '1.6'
  body-md:
    fontFamily: Plus Jakarta Sans
    fontSize: 16px
    fontWeight: '400'
    lineHeight: '1.6'
  label-sm:
    fontFamily: Plus Jakarta Sans
    fontSize: 12px
    fontWeight: '600'
    lineHeight: '1.2'
    letterSpacing: 0.1em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  container-max: 1280px
  stack-unit: 8px
  section-padding: 80px
  gutter: 24px
  margin-page: 40px
---

## Brand & Style

This design system embodies the essence of high-end Indian couture—blending timeless tradition with a clean, contemporary editorial aesthetic. The brand personality is regal yet accessible, sophisticated, and deeply rooted in craftsmanship. 

The visual style is **Minimalist-Luxury**. It leverages generous whitespace (macro-typography) and a restricted, high-contrast color palette to allow product photography to serve as the primary visual driver. The interface uses subtle cues from classical publishing to create a sense of "boutique" exclusivity, moving away from cluttered e-commerce patterns toward a curated, gallery-like experience.

## Colors

The palette is anchored by **Deep Forest Green**, evoking stability and heritage, and **Muted Gold**, used sparingly for high-value accents, iconography, and call-to-action highlights. 

- **Primary:** Forest Green (#123026) is the core brand identifier, used for major buttons, headings, and immersive backgrounds.
- **Secondary:** Gold (#C5A059) provides a metallic warmth for decorative elements and hover states.
- **Surface:** The background utilizes a warm, off-white "Parchment" (#F9F7F2) rather than pure white to soften the digital experience and mimic high-quality stationery.
- **Typography:** Deep charcoal and grays are used for body text to ensure readability without the harshness of pure black.

## Typography

This design system employs a high-contrast typographic pairing to balance tradition and modernity. 

**Playfair Display** (Serif) is reserved for headlines and large display quotes. It carries the weight of the brand's elegance and should be set with slightly tighter tracking in large sizes to emphasize its editorial grace.

**Plus Jakarta Sans** (Sans-Serif) handles all functional and body content. Its modern, geometric roots provide a clean counterpoint to the serif, ensuring the interface feels current and highly legible across devices. Small labels and navigation items should utilize uppercase styling with increased letter spacing to create an "airier" boutique feel.

## Layout & Spacing

The design system utilizes a **Fixed Grid** philosophy for desktop screens to maintain a focused, centered "magazine" feel, transitioning to a fluid model for mobile. 

A 12-column grid provides the structural foundation, but the rhythm is defined by "intentional emptiness." Spacing between major sections should be expansive (80px+) to prevent the premium products from feeling crowded. Content is often organized into asymmetric compositions or centered layouts to break away from standard "row-after-row" e-commerce tropes.

## Elevation & Depth

To maintain a sophisticated and flat "printed matter" aesthetic, this design system avoids heavy drop shadows. Instead, depth is communicated through:

- **Tonal Layering:** Using subtle shifts between the Parchment background and slightly lighter or darker surface containers to define content areas.
- **Low-Contrast Outlines:** Product cards and input fields use ultra-thin (1px) borders in a muted gold or light gray rather than shadows.
- **Overlapping Elements:** Large typography occasionally overlaps imagery to create a sense of Z-axis depth without technical effects.
- **Micro-shadows:** Only used on primary action buttons or floating navigation bars, using a highly diffused, low-opacity (5-10%) tint of the Forest Green color.

## Shapes

The shape language is primarily **Soft** and structured. While the products (ethnic wear) are organic and flowing, the UI containers use subtle 4px (0.25rem) corner radii to provide a sense of architectural precision. 

Buttons and input fields maintain these soft corners, avoiding fully rounded "pill" shapes which can appear too casual for a high-end luxury brand. Product imagery should remain sharp-edged or use the same subtle radius to feel like matted photographs.

## Components

### Buttons
Primary buttons are solid Forest Green with white or gold text, utilizing a rectangular shape with a subtle radius. Secondary buttons should be ghost-style with a thin gold border. All buttons use uppercase label styling for a formal tone.

### Cards
Product cards are minimal, featuring full-bleed imagery. Product names are set in the body-serif style, while prices are set in the sans-serif for clarity. "New Arrival" or "Limited" tags should be small, high-contrast labels in gold.

### Input Fields
Forms utilize "Floating Label" or minimal underline styles to reduce visual clutter. The focus state is indicated by a Forest Green border and a subtle weight change in the label.

### Chips & Filters
Filter tags use a tonal background (Parchment-darker) with no borders, appearing as seamless extensions of the interface.

### Navigation
The main navigation is centered and spacious, using the serif typeface for main categories and sans-serif for utility links (About, Account). A thin gold divider separates the top-bar utility from the primary browsing menu.