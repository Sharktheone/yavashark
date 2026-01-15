package spec

import (
	"bytes"
	"errors"
	"log"
	"net/http"
	"sort"
	"strings"
	"sync"

	"golang.org/x/net/html"
)

const SpecLocation = "https://raw.githubusercontent.com/tc39/ecma262/refs/heads/main/spec.html"
const IntlSpecLocation = "https://tc39.es/ecma402/"
const TemporalSpecLocation = "https://tc39.es/proposal-temporal/"

// Provider holds the parsed ECMAScript specification
type Provider struct {
	Content  map[string]string
	Sections []string
	mu       sync.RWMutex
	loaded   bool
}

var (
	globalProvider *Provider
	once           sync.Once
)

// GetProvider returns the global spec provider, initializing it if needed
func GetProvider() *Provider {
	once.Do(func() {
		globalProvider = &Provider{
			Content: make(map[string]string),
		}
	})
	return globalProvider
}

// Initialize fetches and parses the spec (can be called explicitly or lazily)
func (p *Provider) Initialize() error {
	p.mu.Lock()
	defer p.mu.Unlock()

	if p.loaded {
		return nil
	}

	if p.Content == nil {
		p.Content = make(map[string]string)
	}
	p.Sections = p.Sections[:0]

	log.Println("Initializing ECMAScript spec provider...")

	// Load specs in order of precedence (later specs override earlier for duplicate IDs)
	// 1. Temporal proposal spec
	if err := p.initializeSpec(TemporalSpecLocation); err != nil {
		log.Printf("Warning: Failed to load Temporal spec: %v", err)
		// Continue anyway - it's a proposal, not required
	}

	// 2. Intl spec (ECMA-402)
	if err := p.initializeSpec(IntlSpecLocation); err != nil {
		log.Printf("Warning: Failed to load Intl spec: %v", err)
		// Continue anyway - main spec is more important
	}

	// 3. Main ECMA-262 spec (takes precedence for any duplicate IDs)
	if err := p.initializeSpec(SpecLocation); err != nil {
		return err
	}

	p.loaded = true
	log.Printf("Spec provider initialized with %d sections", len(p.Sections))
	return nil
}

func (p *Provider) initializeSpec(url string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		return errors.New("failed to fetch spec: " + resp.Status)
	}

	root, err := html.Parse(resp.Body)
	if err != nil {
		return err
	}

	var walk func(n *html.Node)
	walk = func(n *html.Node) {
		if n.Type == html.ElementNode {
			for _, a := range n.Attr {
				if a.Key == "id" && a.Val != "" {
					var buf bytes.Buffer
					_ = html.Render(&buf, n)
					id := a.Val

					id = strings.TrimPrefix(id, "sec-")

					p.Sections = append(p.Sections, id)
					p.Content[id] = buf.String()
					break
				}
			}
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			walk(c)
		}
	}
	walk(root)

	return nil
}

func (p *Provider) ensureLoaded() error {
	p.mu.RLock()
	loaded := p.loaded
	p.mu.RUnlock()

	if !loaded {
		return p.Initialize()
	}
	return nil
}

// GetSpec returns the content of a spec section by ID
func (p *Provider) GetSpec(specPath string) (string, error) {
	if err := p.ensureLoaded(); err != nil {
		return "", err
	}

	p.mu.RLock()
	defer p.mu.RUnlock()

	// Normalize the path
	specPath = strings.TrimPrefix(specPath, "sec-")
	specPath = strings.ToLower(specPath)

	if content, exists := p.Content[specPath]; exists {
		return content, nil
	}

	// Try case-insensitive search
	for id, content := range p.Content {
		if strings.EqualFold(id, specPath) {
			return content, nil
		}
	}

	return "", errors.New("spec section not found: " + specPath)
}

// SpecForIntrinsic returns the spec section for a given intrinsic name
func (p *Provider) SpecForIntrinsic(intrinsic string) (string, error) {
	if err := p.ensureLoaded(); err != nil {
		return "", err
	}

	p.mu.RLock()
	defer p.mu.RUnlock()

	// Normalize intrinsic name to section ID format
	normalized := intrinsic
	normalized = strings.TrimPrefix(normalized, "%")
	normalized = strings.TrimSuffix(normalized, "%")
	normalized = strings.ReplaceAll(normalized, "[[", "")
	normalized = strings.ReplaceAll(normalized, "]]", "")
	normalized = strings.ReplaceAll(normalized, "(", "")
	normalized = strings.ReplaceAll(normalized, ")", "")
	normalized = strings.ToLower(normalized)

	if content, exists := p.Content[normalized]; exists {
		return content, nil
	}

	// Try variations
	variations := []string{
		normalized,
		strings.ReplaceAll(normalized, ".", "-"),
		"sec-" + normalized,
	}

	for _, v := range variations {
		if content, exists := p.Content[v]; exists {
			return content, nil
		}
	}

	return "", errors.New("spec section not found for intrinsic: " + intrinsic)
}

// SearchSpec searches spec content for a query and returns matching section IDs
func (p *Provider) SearchSpec(query string) ([]string, error) {
	if err := p.ensureLoaded(); err != nil {
		return nil, err
	}

	p.mu.RLock()
	defer p.mu.RUnlock()

	query = strings.ToLower(query)
	matches := make([]string, 0)

	for id, content := range p.Content {
		if strings.Contains(strings.ToLower(content), query) {
			matches = append(matches, id)
		}
	}

	sort.Strings(matches)
	return matches, nil
}

// SearchSections searches section IDs/titles for a query
func (p *Provider) SearchSections(query string) ([]string, error) {
	if err := p.ensureLoaded(); err != nil {
		return nil, err
	}

	p.mu.RLock()
	defer p.mu.RUnlock()

	query = strings.ToLower(query)
	matches := make([]string, 0)

	for _, id := range p.Sections {
		if strings.Contains(strings.ToLower(id), query) {
			matches = append(matches, id)
		}
	}

	sort.Strings(matches)
	return matches, nil
}
