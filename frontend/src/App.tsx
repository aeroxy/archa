import { useState, useEffect, useMemo } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

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

interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  thinking?: string;
  timestamp: string;
}

const convertToMessages = (jsonl: string): Message[] => {
  return jsonl.split('\n')
    .filter(line => line.trim())
    .map(line => {
      try {
        const obj = JSON.parse(line);
        if (obj.type === 'user') {
          const content = obj.message.content;
          const text = typeof content === 'string' ? content : (Array.isArray(content) ? content.map((c: any) => c.text).join('\n') : '');
          return { role: 'user', content: text, timestamp: obj.timestamp };
        } else if (obj.type === 'assistant') {
          const content = obj.message.content;
          let text = '';
          let thinking = '';
          if (Array.isArray(content)) {
            text = content.filter((c: any) => c.type === 'text').map((c: any) => c.text).join('\n');
            thinking = content.filter((c: any) => c.type === 'thinking').map((c: any) => c.thinking).join('\n');
          } else if (typeof content === 'string') {
            text = content;
          }
          return { role: 'assistant', content: text, thinking, timestamp: obj.timestamp };
        }
        return null;
      } catch (e) {
        return null;
      }
    })
    .filter((m): m is Message => m !== null);
}

function App() {
  const [projects, setProjects] = useState<Project[]>([])
  const [expandedProjects, setExpandedProjects] = useState<Record<string, boolean>>({})
  const [projectSessions, setProjectSessions] = useState<Record<string, Session[]>>({})
  const [selectedSession, setSelectedSession] = useState<{ project_id: string, id: string } | null>(null)
  const [conversation, setConversation] = useState<string>('')
  const [recentSessions, setRecentSessions] = useState<Session[]>([])

  useEffect(() => {
    fetch('/api/projects')
      .then(res => res.json())
      .then(data => setProjects(data))
    
    fetch('/api/recent-sessions')
      .then(res => res.json())
      .then(data => setRecentSessions(data))
  }, [])

  const toggleProject = async (projectId: string) => {
    const isExpanded = !!expandedProjects[projectId];
    setExpandedProjects(prev => ({ ...prev, [projectId]: !isExpanded }));

    if (!isExpanded && !projectSessions[projectId]) {
      const res = await fetch(`/api/sessions/${projectId}`);
      const data = await res.json();
      setProjectSessions(prev => ({ ...prev, [projectId]: data }));
    }
  }

  useEffect(() => {
    if (selectedSession) {
      fetch(`/api/session/${selectedSession.project_id}/${selectedSession.id}`)
        .then(res => res.text())
        .then(data => setConversation(data))
    }
  }, [selectedSession])

  const messages = useMemo(() => convertToMessages(conversation), [conversation]);

  const exportMarkdown = () => {
    if (!selectedSession) return;
    const md = messages.map(m => `### ${m.role === 'user' ? 'You' : 'Claude'}\n\n${m.thinking ? `> Thinking: ${m.thinking}\n\n` : ''}${m.content}`).join('\n\n');
    const blob = new Blob([md], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${selectedSession.id}.md`;
    a.click();
  }

  return (
    <div className="flex h-screen w-full bg-background">
      {/* Column 1: COI Switcher */}
      <aside className="w-16 border-r border-outline-variant bg-surface-container-low flex flex-col items-center py-4 gap-4">
        <div className="px-2.5"><img src="/logo.svg" alt="Archa Logo" className="w-8 h-8" /></div>
        <button className="w-12 h-12 flex flex-col items-center justify-center text-primary bg-primary-container/10 border-l-2 border-primary">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" className="w-5 h-5 fill-current">
            <path d="m19.6 66.5 19.7-11 .3-1-.3-.5h-1l-3.3-.2-11.2-.3L14 53l-9.5-.5-2.4-.5L0 49l.2-1.5 2-1.3 2.9.2 6.3.5 9.5.6 6.9.4L38 49.1h1.6l.2-.7-.5-.4-.4-.4L29 41l-10.6-7-5.6-4.1-3-2-1.5-2-.6-4.2 2.7-3 3.7.3.9.2 3.7 2.9 8 6.1L37 36l1.5 1.2.6-.4.1-.3-.7-1.1L33 25l-6-10.4-2.7-4.3-.7-2.6c-.3-1-.4-2-.4-3l3-4.2L28 0l4.2.6L33.8 2l2.6 6 4.1 9.3L47 29.9l2 3.8 1 3.4.3 1h.7v-.5l.5-7.2 1-8.7 1-11.2.3-3.2 1.6-3.8 3-2L61 2.6l2 2.9-.3 1.8-1.1 7.7L59 27.1l-1.5 8.2h.9l1-1.1 4.1-5.4 6.9-8.6 3-3.5L77 13l2.3-1.8h4.3l3.1 4.7-1.4 4.9-4.4 5.6-3.7 4.7-5.3 7.1-3.2 5.7.3.4h.7l12-2.6 6.4-1.1 7.6-1.3 3.5 1.6.4 1.6-1.4 3.4-8.2 2-9.6 2-14.3 3.3-.2.1.2.3 6.4.6 2.8.2h6.8l12.6 1 3.3 2 1.9 2.7-.3 2-5.1 2.6-6.8-1.6-16-3.8-5.4-1.3h-.8v.4l4.6 4.5 8.3 7.5L89 80.1l.5 2.4-1.3 2-1.4-.2-9.2-7-3.6-3-8-6.8h-.5v.7l1.8 2.7 9.8 14.7.5 4.5-.7 1.4-2.6 1-2.7-.6-5.8-8-6-9-4.7-8.2-.5.4-2.9 30.2-1.3 1.5-3 1.2-2.5-2-1.4-3 1.4-6.2 1.6-8 1.3-6.4 1.2-7.9.7-2.6v-.2H49L43 72l-9 12.3-7.2 7.6-1.7.7-3-1.5.3-2.8L24 86l10-12.8 6-7.9 4-4.6-.1-.5h-.3L17.2 77.4l-4.7.6-2-2 .2-3 1-1 8-5.5Z"></path>
          </svg>
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
                  onClick={() => setSelectedSession({ project_id: project.id, id: session.id })}
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
              messages.map((m, i) => (
                <div key={i} className={`mb-12 ${m.role === 'user' ? '' : ''}`}>
                  <div className="flex items-center gap-3 mb-4">
                    <div className={`w-6 h-6 rounded-full flex items-center justify-center font-bold text-[10px] ${m.role === 'user' ? 'bg-surface-container-highest text-outline' : 'bg-primary text-white'}`}>
                      {m.role === 'user' ? 'U' : (
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" className="w-4 h-4 fill-current">
                          <path d="m19.6 66.5 19.7-11 .3-1-.3-.5h-1l-3.3-.2-11.2-.3L14 53l-9.5-.5-2.4-.5L0 49l.2-1.5 2-1.3 2.9.2 6.3.5 9.5.6 6.9.4L38 49.1h1.6l.2-.7-.5-.4-.4-.4L29 41l-10.6-7-5.6-4.1-3-2-1.5-2-.6-4.2 2.7-3 3.7.3.9.2 3.7 2.9 8 6.1L37 36l1.5 1.2.6-.4.1-.3-.7-1.1L33 25l-6-10.4-2.7-4.3-.7-2.6c-.3-1-.4-2-.4-3l3-4.2L28 0l4.2.6L33.8 2l2.6 6 4.1 9.3L47 29.9l2 3.8 1 3.4.3 1h.7v-.5l.5-7.2 1-8.7 1-11.2.3-3.2 1.6-3.8 3-2L61 2.6l2 2.9-.3 1.8-1.1 7.7L59 27.1l-1.5 8.2h.9l1-1.1 4.1-5.4 6.9-8.6 3-3.5L77 13l2.3-1.8h4.3l3.1 4.7-1.4 4.9-4.4 5.6-3.7 4.7-5.3 7.1-3.2 5.7.3.4h.7l12-2.6 6.4-1.1 7.6-1.3 3.5 1.6.4 1.6-1.4 3.4-8.2 2-9.6 2-14.3 3.3-.2.1.2.3 6.4.6 2.8.2h6.8l12.6 1 3.3 2 1.9 2.7-.3 2-5.1 2.6-6.8-1.6-16-3.8-5.4-1.3h-.8v.4l4.6 4.5 8.3 7.5L89 80.1l.5 2.4-1.3 2-1.4-.2-9.2-7-3.6-3-8-6.8h-.5v.7l1.8 2.7 9.8 14.7.5 4.5-.7 1.4-2.6 1-2.7-.6-5.8-8-6-9-4.7-8.2-.5.4-2.9 30.2-1.3 1.5-3 1.2-2.5-2-1.4-3 1.4-6.2 1.6-8 1.3-6.4 1.2-7.9.7-2.6v-.2H49L43 72l-9 12.3-7.2 7.6-1.7.7-3-1.5.3-2.8L24 86l10-12.8 6-7.9 4-4.6-.1-.5h-.3L17.2 77.4l-4.7.6-2-2 .2-3 1-1 8-5.5Z"></path>
                        </svg>
                      )}
                    </div>
                    <span className="font-meta-label text-[10px] text-outline uppercase">
                      {m.role === 'user' ? 'You' : 'Claude'} • {new Date(m.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                  {m.thinking && (
                    <div className="text-xs text-outline italic border-l-2 border-outline-variant pl-4 mb-4 bg-surface-container-low/50 py-2 rounded-r">
                      {m.thinking}
                    </div>
                  )}
                  <div className={`prose prose-slate max-w-none font-reader-body text-reader-body leading-relaxed text-on-surface ${m.role === 'user' ? 'italic' : ''}`}>
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>{m.content}</ReactMarkdown>
                  </div>
                </div>
              ))
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
                        onClick={() => setSelectedSession({ project_id: s.project_id, id: s.id })}
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
          <p className="text-[10px] text-outline uppercase tracking-wider">Archa v0.1.1 • CLI Session Chronicle</p>
        </div>
      </section>
    </div>
  )
}

export default App
