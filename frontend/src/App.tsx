import { useState, useEffect, useMemo } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { ClaudeIcon, OpencodeIcon } from './icons'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { 
  Wrench, 
  CheckCircle2, 
  AlertCircle, 
  ChevronRight, 
  ChevronDown,
  Bot,
  Sparkles
} from 'lucide-react'

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

interface Project {
  id: string;
  name: string;
  cwd?: string;
}

interface Session {
  id: string;
  project_id: string;
  title: string;
}

type ContentBlock = 
  | { type: 'text'; text: string }
  | { type: 'thinking'; thinking: string; signature?: string }
  | { type: 'tool_use'; id: string; name: string; input: any }
  | { type: 'tool_result'; tool_use_id: string; content: any; is_error?: boolean };

interface Message {
  role: 'user' | 'assistant' | 'system';
  content: ContentBlock[];
  timestamp: string;
}

const convertToMessages = (jsonl: string): Message[] => {
  const rawEntries = jsonl.split('\n')
    .filter(line => line.trim())
    .map(line => {
      try {
        const obj = JSON.parse(line);
        if (obj.type === 'user' || obj.type === 'assistant') {
          const content = obj.message.content;
          let blocks: ContentBlock[] = [];
          
          if (typeof content === 'string') {
            blocks = [{ type: 'text', text: content }];
          } else if (Array.isArray(content)) {
            blocks = content.map((c: any) => {
              if (c.type === 'text') return { type: 'text', text: typeof c.text === 'string' ? c.text : (c.text?.text || JSON.stringify(c.text)) };
              if (c.type === 'thinking') return { type: 'thinking', thinking: c.thinking, signature: c.signature };
              if (c.type === 'tool_use') return { type: 'tool_use', id: c.id, name: c.name, input: c.input };
              if (c.type === 'tool_result') return { type: 'tool_result', tool_use_id: c.tool_use_id, content: c.content, is_error: c.is_error };
              return { type: 'text', text: JSON.stringify(c) };
            });
          }
          
          return { 
            role: obj.type as 'user' | 'assistant' | 'system', 
            content: blocks, 
            timestamp: obj.timestamp,
            msgId: obj.message.id || obj.promptId || obj.uuid 
          };
        }
        return null;
      } catch (e) {
        return null;
      }
    })
    .filter((m): m is any => m !== null);

  // 1. Collect all tool results from the entire session
  const toolResultsMap = new Map<string, ContentBlock>();
  rawEntries.forEach(entry => {
    entry.content.forEach((block: ContentBlock) => {
      if (block.type === 'tool_result') {
        toolResultsMap.set(block.tool_use_id, block);
      }
    });
  });

  // 2. Clean entries: remove tool_result from user messages and attach to tool_use in assistant messages
  rawEntries.forEach(entry => {
    // Remove tool results from their original user message
    entry.content = entry.content.filter((block: ContentBlock) => block.type !== 'tool_result');
    
    // For assistant messages, find tool_use and append their results if they exist
    if (entry.role === 'assistant') {
      const newContent: ContentBlock[] = [];
      entry.content.forEach((block: ContentBlock) => {
        newContent.push(block);
        if (block.type === 'tool_use') {
          const result = toolResultsMap.get(block.id);
          if (result) {
            newContent.push(result);
          }
        }
      });
      entry.content = newContent;
    }
  });

  const merged: Message[] = [];
  const msgMap = new Map<string, number>();

  for (const entry of rawEntries) {
    // Skip entries that became empty after moving tool results
    if (entry.content.length === 0) continue;

    const existingIdx = msgMap.get(entry.msgId);
    
    if (existingIdx !== undefined) {
      const existing = merged[existingIdx];
      const mergedBlocks = [...existing.content];
      
      entry.content.forEach((newBlock: ContentBlock) => {
        const sameTypeIdx = mergedBlocks.findIndex(b => {
          if (b.type === 'thinking' && newBlock.type === 'thinking') return true;
          if (b.type === 'tool_use' && newBlock.type === 'tool_use' && b.id === newBlock.id) return true;
          if (b.type === 'tool_result' && newBlock.type === 'tool_result' && b.tool_use_id === newBlock.tool_use_id) return true;
          return false;
        });

        if (sameTypeIdx !== -1) {
          mergedBlocks[sameTypeIdx] = newBlock;
        } else {
          mergedBlocks.push(newBlock);
        }
      });
      
      existing.content = mergedBlocks;
      existing.timestamp = entry.timestamp;
    } else {
      msgMap.set(entry.msgId, merged.length);
      merged.push({
        role: entry.role,
        content: entry.content,
        timestamp: entry.timestamp
      });
    }
  }
  return merged;
}

function App() {
  const [activeCli, setActiveCli] = useState<'claude' | 'opencode'>('claude');
  const [projects, setProjects] = useState<Project[]>([])
  const [expandedProjects, setExpandedProjects] = useState<Record<string, boolean>>({})
  const [projectSessions, setProjectSessions] = useState<Record<string, Session[]>>({})
  const [selectedSession, setSelectedSession] = useState<{ cli: string, project_id: string, id: string } | null>(null)
  const [conversation, setConversation] = useState<string>('')
  const [recentSessions, setRecentSessions] = useState<Session[]>([])

  // Helper to fetch and expand a project's sessions
  const expandProject = async (cli: string, projectId: string) => {
    setExpandedProjects(prev => ({ ...prev, [projectId]: true }));
    const res = await fetch(`/api/${cli}/sessions/${projectId}`);
    const data = await res.json();
    setProjectSessions(prev => ({ ...prev, [projectId]: data }));
  };

  // Initial load / Browser navigation handler
  useEffect(() => {
    const handleLocationChange = async () => {
      const match = window.location.pathname.match(/^\/([^\/]+)\/([^\/]+)$/);
      if (match) {
        const cliParam = match[1] as 'claude' | 'opencode';
        const sessionId = match[2];
        
        if (cliParam === 'claude' || cliParam === 'opencode') {
          setActiveCli(cliParam);
          
          try {
            // Find which project this session belongs to
            const infoRes = await fetch(`/api/${cliParam}/session-info/${sessionId}`);
            if (infoRes.ok) {
              const { project_id } = await infoRes.json();
              setSelectedSession({ cli: cliParam, project_id, id: sessionId });
              expandProject(cliParam, project_id);
            } else {
              setSelectedSession(null);
            }
          } catch (e) {
            console.error('Failed to find session info', e);
            setSelectedSession(null);
          }
        }
      } else {
        setSelectedSession(null);
      }
    };

    handleLocationChange();

    window.addEventListener('popstate', handleLocationChange);
    return () => window.removeEventListener('popstate', handleLocationChange);
  }, []);

  // Fetch projects and recent sessions when CLI changes
  useEffect(() => {
    setProjects([]);
    setRecentSessions([]);
    setExpandedProjects({});
    
    fetch(`/api/${activeCli}/projects`)
      .then(res => res.json())
      .then(data => setProjects(data));
    
    fetch(`/api/${activeCli}/recent-sessions`)
      .then(res => res.json())
      .then(data => setRecentSessions(data));
  }, [activeCli]);

  const toggleProject = async (projectId: string) => {
    const isExpanded = !!expandedProjects[projectId];
    
    if (isExpanded) {
      setExpandedProjects(prev => ({ ...prev, [projectId]: false }));
    } else {
      await expandProject(activeCli, projectId);
    }
  }

  // Handle session selection from UI (Updates URL)
  const handleSessionSelect = (project_id: string, id: string) => {
    setSelectedSession({ cli: activeCli, project_id, id });
    const newPath = `/${activeCli}/${id}`;
    if (window.location.pathname !== newPath) {
      window.history.pushState(null, '', newPath);
    }
  };

  // Fetch conversation when selection changes
  useEffect(() => {
    if (selectedSession) {
      fetch(`/api/${selectedSession.cli}/session/${selectedSession.project_id}/${selectedSession.id}`)
        .then(res => res.text())
        .then(data => setConversation(data))
    }
  }, [selectedSession])

  const messages = useMemo(() => convertToMessages(conversation), [conversation]);

  const exportMarkdown = () => {
    if (!selectedSession) return;
    const md = messages.map(m => {
      const header = `### ${m.role === 'user' ? 'User' : (selectedSession.cli === 'opencode' ? 'Opencode' : 'Claude')}\n\n`;
      const content = m.content.map(block => {
        if (block.type === 'text') return block.text.trim();
        if (block.type === 'thinking') return `> Thinking: ${block.thinking.trim()}`;
        if (block.type === 'tool_use') return `#### Tool Use: ${block.name}\n\`\`\`json\n${JSON.stringify(block.input, null, 2)}\n\`\`\``;
        if (block.type === 'tool_result') return `#### Tool Result: ${block.tool_use_id}\n\`\`\`\n${(typeof block.content === 'string' ? block.content : JSON.stringify(block.content, null, 2)).trim()}\n\`\`\``;
        return '';
      }).filter(b => b !== '').join('\n\n');
      return header + content;
    }).join('\n\n---\n\n');
    
    const blob = new Blob([md], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${selectedSession.id}.md`;
    a.click();
  }

  const ThinkingWidget = ({ thinking }: { thinking: string }) => {
    const [isExpanded, setIsExpanded] = useState(false);
    return (
      <div className="rounded-lg border border-outline-variant bg-surface-container-low/30 overflow-hidden mb-4">
        <button 
          onClick={() => setIsExpanded(!isExpanded)} 
          className="w-full px-4 py-2 flex items-center justify-between hover:bg-surface-container-low transition-colors"
        >
          <div className="flex items-center gap-2">
            <div className="relative">
              <Bot className="h-4 w-4 text-outline" />
              <Sparkles className="h-2.5 w-2.5 text-outline absolute -top-1 -right-1" />
            </div>
            <span className="text-xs font-medium text-outline italic">
              Thinking...
            </span>
          </div>
          {isExpanded ? <ChevronDown className="h-4 w-4 text-outline" /> : <ChevronRight className="h-4 w-4 text-outline" />}
        </button>
        {isExpanded && (
          <div className="px-4 pb-4 pt-2 border-t border-outline-variant">
            <pre className="text-[11px] font-mono text-outline whitespace-pre-wrap italic">
              {thinking.trim()}
            </pre>
          </div>
        )}
      </div>
    );
  };

  const renderContent = (content: ContentBlock[]) => {
    return content.map((block, idx) => {
      if (block.type === 'text') {
        if (!block.text.trim()) return null;
        return (
          <div key={idx} className="prose prose-slate max-w-none font-reader-body text-reader-body leading-relaxed text-on-surface mb-4">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{block.text}</ReactMarkdown>
          </div>
        );
      }
      
      if (block.type === 'thinking') {
        return <ThinkingWidget key={idx} thinking={block.thinking} />;
      }
      
      if (block.type === 'tool_use') {
        return (
          <div key={idx} className="rounded-lg border border-primary/20 bg-primary/5 p-3 mb-4">
            <div className="flex items-center gap-2 mb-2">
              <Wrench className="h-4 w-4 text-primary" />
              <span className="text-xs font-semibold uppercase tracking-wider text-primary">Tool Use: {block.name}</span>
            </div>
            <pre className="text-[11px] font-mono bg-white/50 p-2 rounded border border-primary/10 overflow-x-auto">
              {JSON.stringify(block.input, null, 2)}
            </pre>
          </div>
        );
      }
      
      if (block.type === 'tool_result') {
        const isError = block.is_error;
        const resultText = typeof block.content === 'string' 
          ? block.content 
          : (block.content?.text || JSON.stringify(block.content, null, 2));
        
        if (typeof resultText === 'string' && !resultText.trim()) return null;
          
        return (
          <div key={idx} className={cn(
            "rounded-lg border p-3 mb-4",
            isError ? "border-error/20 bg-error/5" : "border-success/20 bg-success/5"
          )}>
            <div className="flex items-center gap-2 mb-2">
              {isError ? <AlertCircle className="h-4 w-4 text-error" /> : <CheckCircle2 className="h-4 w-4 text-success" />}
              <span className={cn(
                "text-xs font-semibold uppercase tracking-wider",
                isError ? "text-error" : "text-success"
              )}>
                Tool Result
              </span>
            </div>
            <pre className="text-[11px] font-mono bg-white/50 p-2 rounded border border-outline-variant overflow-x-auto whitespace-pre-wrap">
              {resultText}
            </pre>
          </div>
        );
      }
      
      return null;
    });
  };

  return (
    <div className="flex h-screen w-full bg-background">
      {/* Column 1: COI Switcher */}
      <aside className="w-16 border-r border-outline-variant bg-surface-container-low flex flex-col items-center py-4 gap-4">
        <div className="px-2.5"><img src="/logo.svg" alt="Archa Logo" className="w-8 h-8" /></div>
        
        <button 
          onClick={() => setActiveCli('claude')}
          className={`w-12 h-12 flex flex-col items-center justify-center transition-colors ${activeCli === 'claude' ? 'text-primary bg-primary-container/10 border-l-2 border-primary' : 'text-on-surface-variant hover:bg-surface-container border-l-2 border-transparent'}`}
        >
          <ClaudeIcon />
          <span className="text-[10px] mt-1">Claude</span>
        </button>

      </aside>

      {/* Column 2: Projects & Sessions */}
      <main className="w-sidebar-width border-r border-outline-variant bg-surface-container-lowest flex flex-col overflow-hidden">
        <header className="h-12 flex items-center px-4 border-b border-outline-variant shrink-0">
          <h2 className="font-list-title text-sm text-on-surface">Explorer</h2>
        </header>
        <div className="flex-1 overflow-y-auto custom-scrollbar">
          {projects.map(project => (
            <div key={project.id}>
              <button 
                onClick={() => toggleProject(project.id)}
                className="w-full flex items-center px-2 py-1.5 hover:bg-surface-container-high text-left text-xs font-medium text-on-surface-variant"
                title={project.cwd}
              >
                <span className="material-symbols-outlined text-sm mr-1">
                  {expandedProjects[project.id] ? 'expand_more' : 'chevron_right'}
                </span>
                <span className="truncate">{project.name}</span>
              </button>
              {expandedProjects[project.id] && projectSessions[project.id]?.map(session => (
                <button
                  key={session.id}
                  onClick={() => handleSessionSelect(project.id, session.id)}
                  className={`w-full pl-8 pr-2 py-1.5 text-left text-xs truncate transition-colors ${selectedSession?.id === session.id ? 'bg-primary-container/10 text-primary border-r-2 border-primary' : 'text-on-surface-variant hover:bg-surface-container'}`}
                >
                  {session.title}
                </button>
              ))}
            </div>
          ))}
        </div>
      </main>

      {/* Column 3: Reader View */}
      <section className="flex-1 flex flex-col bg-white overflow-hidden relative">
        <header className="h-12 flex justify-between items-center px-6 border-b border-outline-variant bg-white/80 backdrop-blur-md sticky top-0 z-10">
          <div className="flex items-center gap-4">
            <h1 className="font-nav-item text-sm font-semibold truncate max-w-md">
              {selectedSession ? projectSessions[selectedSession.project_id]?.find(s => s.id === selectedSession.id)?.title : 'Recent Conversations'}
            </h1>
          </div>
          <div className="flex items-center gap-2">
            {selectedSession && (
              <button 
                onClick={exportMarkdown}
                className="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-outline-variant hover:bg-surface-container-low transition-all text-xs font-medium"
              >
                <span className="material-symbols-outlined text-base">download</span>
                Export MD
              </button>
            )}
          </div>
        </header>

        <div className="flex-1 overflow-y-auto custom-scrollbar p-8">
          <div className="max-w-reader-max-width mx-auto pb-24">
            {selectedSession ? (
              (() => {
                const renderedMessages = messages.map((m, i) => {
                  const renderedBlocks = renderContent(m.content);
                  if (renderedBlocks.every(block => block === null)) return null;

                  return (
                    <div key={i} className={`mb-12 ${m.role === 'user' ? '' : ''}`}>
                      <div className="flex items-center gap-3 mb-4">
                        <div className={`w-6 h-6 rounded-full flex items-center justify-center font-bold text-[10px] ${m.role === 'user' ? 'bg-surface-container-highest text-outline' : 'bg-primary text-white'}`}>
                          {m.role === 'user' ? 'U' : (
                            selectedSession.cli === 'opencode' ? <OpencodeIcon /> : <ClaudeIcon />
                          )}
                        </div>
                        <span className="font-meta-label text-[10px] text-outline uppercase">
                          {m.role === 'user' ? 'User' : (selectedSession.cli === 'opencode' ? 'Opencode' : 'Claude')} • {new Date(m.timestamp).toLocaleTimeString()}
                        </span>
                      </div>
                      <div className={cn(
                        "max-w-none",
                        m.role === 'user' ? "italic" : ""
                      )}>
                        {renderedBlocks}
                      </div>
                    </div>
                  );
                }).filter(m => m !== null);

                return renderedMessages.length > 0 ? (
                  renderedMessages
                ) : (
                  <div className="flex flex-col items-center justify-center h-full text-outline py-20">
                    <span className="material-symbols-outlined text-6xl mb-4">hourglass_empty</span>
                    <p>This session is empty.</p>
                  </div>
                );
              })()
            ) : (
              <div className="flex flex-col items-center justify-center h-full text-outline py-20">
                <span className="material-symbols-outlined text-6xl mb-4">chat_bubble_outline</span>
                <p>Select a conversation to start reading</p>
                {recentSessions.length > 0 && (
                  <div className="mt-8 w-full max-w-md">
                    <h3 className="text-center text-sm font-semibold mb-4 text-on-surface">Most Recent</h3>
                    {recentSessions.map(s => (
                      <button 
                        key={s.id}
                        onClick={() => handleSessionSelect(s.project_id, s.id)}
                        className="w-full p-4 mb-2 bg-surface-container-lowest border border-outline-variant rounded-lg hover:border-primary transition-colors text-left"
                      >
                        <div className="text-sm font-medium text-on-surface mb-1">{s.title}</div>
                        <div className="text-[10px] text-outline uppercase">{s.project_id}</div>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
        <div className="p-4 border-t border-outline-variant bg-surface-container-low text-center">
          <p className="text-[10px] text-outline uppercase tracking-wider">Archa v0.1.2 • CLI Session Chronicle</p>
        </div>
      </section>
    </div>
  )
}

export default App
