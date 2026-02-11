package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// Exit codes
const (
	ExitOK    = 0
	ExitFound = 1
	ExitError = 2
)

func main() {
	if err := run(); err != nil {
		fmt.Fprintf(os.Stderr, "ERROR: %v\n", err)
		os.Exit(ExitError)
	}
}

func run() error {
	// 1. Fast Check: Scan Cargo.lock
	found, err := checkCargoLock()
	if err != nil {
		return fmt.Errorf("checking Cargo.lock: %w", err)
	}
	if !found {
		// Not found, all good.
		return nil
	}

	// 2. Found forbidden crate. Analyze.
	fmt.Fprintf(os.Stderr, "FOUND forbidden crate: sqlx-mysql\n")

	// 3. Try fully precise metadata trace first
	if err := traceViaMetadata(); err != nil {
		fmt.Fprintf(os.Stderr, "NOTE: Metadata trace failed or incomplete: %v\n", err)
		fmt.Fprintf(os.Stderr, "Falling back to Cargo.lock analysis...\n")
		// 4. Fallback to Cargo.lock trace
		if err := traceViaLock(); err != nil {
			return fmt.Errorf("lock trace also failed: %w", err)
		}
	}

	printRemediation()
	os.Exit(ExitFound)
	return nil
}

func checkCargoLock() (bool, error) {
	f, err := os.Open("Cargo.lock")
	if err != nil {
		if os.IsNotExist(err) {
			return false, fmt.Errorf("Cargo.lock not found in current directory")
		}
		return false, err
	}
	defer f.Close()

	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == `name = "sqlx-mysql"` {
			return true, nil
		}
	}
	return false, scanner.Err()
}

// --- Metadata Trace ---

type Metadata struct {
	Resolve struct {
		Nodes []Node `json:"nodes"`
	} `json:"resolve"`
	WorkspaceMembers []string  `json:"workspace_members"`
	Packages         []Package `json:"packages"`
}

type Package struct {
	Name string `json:"name"`
	ID   string `json:"id"`
}

type Node struct {
	ID       string   `json:"id"`
	Deps     []Dep    `json:"deps"`
	Features []string `json:"features"`
}

type Dep struct {
	Name string `json:"name"`
	Pkg  string `json:"pkg"` // ID of the dependency
}

func traceViaMetadata() error {
	cmd := exec.Command("cargo", "metadata", "--locked", "--format-version", "1")
	cmd.Stderr = os.Stderr
	out, err := cmd.Output()
	if err != nil {
		return err
	}

	var meta Metadata
	if err := json.Unmarshal(out, &meta); err != nil {
		return err
	}

	// Build ID -> Name map
	idToName := make(map[string]string, len(meta.Packages))
	for _, p := range meta.Packages {
		idToName[p.ID] = p.Name
	}

	// Build graph: Reversed edges (Child -> []Parent)
	parents := make(map[string][]string)
	nodeMap := make(map[string]Node)

	var targetID string

	// Set of workspace members for quick lookup
	members := make(map[string]bool)
	for _, m := range meta.WorkspaceMembers {
		members[m] = true
	}

	for _, n := range meta.Resolve.Nodes {
		nodeMap[n.ID] = n
		// Use the map for name resolution
		pkgName := idToName[n.ID]
		if pkgName == "" {
			// Fallback if not found (unlikely in valid metadata)
			pkgName = n.ID
		}

		if pkgName == "sqlx-mysql" {
			targetID = n.ID
		}

		for _, d := range n.Deps {
			parents[d.Pkg] = append(parents[d.Pkg], n.ID)
		}
	}

	if targetID == "" {
		return fmt.Errorf("sqlx-mysql not found in metadata (but was in lock?)")
	}

	// BFS to find shortest path to any workspace member
	type pathItem struct {
		id   string
		path []string
	}

	queue := []pathItem{{id: targetID, path: []string{targetID}}}
	visited := make(map[string]bool)
	visited[targetID] = true

	for len(queue) > 0 {
		curr := queue[0]
		queue = queue[1:]

		if members[curr.id] {
			// Found path!
			printMetadataPath(curr.path, nodeMap, idToName)
			return nil
		}

		for _, pID := range parents[curr.id] {
			if !visited[pID] {
				visited[pID] = true
				newPath := make([]string, len(curr.path)+1)
				copy(newPath, curr.path)
				newPath[len(curr.path)] = pID // Append parent
				queue = append(queue, pathItem{id: pID, path: newPath})
			}
		}
	}

	return fmt.Errorf("no path found from sqlx-mysql to workspace members")
}

func getPkgName(id string) string {
	// Keep for lock trace fallback
	parts := strings.Fields(id)
	if len(parts) > 0 {
		return parts[0]
	}
	return id
}

func printMetadataPath(path []string, nodes map[string]Node, idToName map[string]string) {
	fmt.Println("\nMETADATA TRACE:")
	// Path is [sqlx-mysql, parent, grandparent, ..., workspace_member]
	// We want to print: Workspace Member -> ... -> sqlx-mysql
	for i := len(path) - 1; i >= 0; i-- {
		id := path[i]
		node := nodes[id]
		name := idToName[id]
		if name == "" {
			name = id
		}

		pad := strings.Repeat("  ", len(path)-1-i)
		arrow := ""
		if i < len(path)-1 {
			arrow = "-> "
		}

		fmt.Printf("%s%s%s\n", pad, arrow, name)
		if len(node.Features) > 0 {
			// Filter for relevant features if possible, or just list enabled ones
			// For brevity, just list first few or all if short
			feats := strings.Join(node.Features, ", ")
			if len(feats) > 100 {
				feats = feats[:97] + "..."
			}
			fmt.Printf("%s    (features: [%s])\n", pad, feats)
		}
	}
}

// --- Lock Trace (Fallback) ---
// Simple parsing of [[package]] blocks

func traceViaLock() error {
	// This is a much rougher parser, assuming standard TOML format without a real TOML parser to stay stdlib
	f, err := os.Open("Cargo.lock")
	if err != nil {
		return err
	}
	defer f.Close()

	// We need to build a graph name -> []name (parents)
	// [[package]]
	// name = "foo"
	// dependencies = [ "bar", "baz" ]

	parents := make(map[string][]string)

	scanner := bufio.NewScanner(f)
	var currentPkg string
	inDeps := false

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "[[package]]" {
			currentPkg = ""
			inDeps = false
			continue
		}

		if strings.HasPrefix(line, `name = "`) {
			val := strings.TrimPrefix(line, `name = "`)
			val = strings.TrimSuffix(val, `"`)
			currentPkg = val
			continue
		}

		if line == "dependencies = [" {
			inDeps = true
			continue
		}

		if inDeps {
			if line == "]" {
				inDeps = false
				continue
			}
			// Parse dependency line: "name", or "name version", etc.
			// usually: "pkg-name", or "pkg-name version"
			depLine := strings.Trim(line, `",`)
			// If it has version info, it might look like "bitflags 2.4.1"
			// users might just use "bitflags"
			parts := strings.Fields(depLine)
			if len(parts) > 0 {
				depName := parts[0]
				if currentPkg != "" {
					parents[depName] = append(parents[depName], currentPkg)
				}
			}
		}
	}

	// BFS
	target := "sqlx-mysql"
	// Need to know workspace members.
	// Without metadata, we can guess or just find *any* path that ends in a "known" source?
	// Or we can just print the tree upwards up to a certain depth.
	// Since we are in `veil-rs`, members likely start with `veil-`.

	type pathItem struct {
		name string
		path []string
	}

	queue := []pathItem{{name: target, path: []string{target}}}
	visited := make(map[string]bool)
	visited[target] = true

	// Limit iterations
	iters := 0
	foundPath := false

	for len(queue) > 0 && iters < 10000 {
		iters++
		curr := queue[0]
		queue = queue[1:]

		if strings.HasPrefix(curr.name, "veil-") {
			printLockPath(curr.path)
			foundPath = true
			break // Show one
		}

		for _, p := range parents[curr.name] {
			if !visited[p] {
				visited[p] = true
				newPath := make([]string, len(curr.path)+1)
				copy(newPath, curr.path)
				newPath[len(curr.path)] = p
				queue = append(queue, pathItem{name: p, path: newPath})
			}
		}
	}

	if !foundPath {
		fmt.Println("\nLOCK TRACE (Partial - could not trace to veil-*):")
		// Just dump immediate parents of sqlx-mysql
		fmt.Printf("Parents of %s: %v\n", target, parents[target])
	}

	return nil
}

func printLockPath(path []string) {
	fmt.Println("\nLOCK TRACE:")
	for i := len(path) - 1; i >= 0; i-- {
		name := path[i]
		pad := strings.Repeat("  ", len(path)-1-i)
		arrow := ""
		if i < len(path)-1 {
			arrow = "-> "
		}
		fmt.Printf("%s%s%s\n", pad, arrow, name)
	}
}

func printRemediation() {
	fmt.Println("\nREMEDIATION:")
	fmt.Println("1. Identify the 'veil-*' crate in the trace provided above.")
	fmt.Println("2. Check its Cargo.toml for 'sqlx' features.")
	fmt.Println("3. Ensure 'default-features = false' is set for 'sqlx'.")
	fmt.Println("4. Explicitly Disable 'mysql', 'all-databases', or 'any' features.")
	fmt.Println("5. Run 'cargo update -p <dependency>' to refresh Cargo.lock.")
}
