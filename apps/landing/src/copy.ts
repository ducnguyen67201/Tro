export type DemoPhase =
  | "idle"
  | "targeting"
  | "listening"
  | "thinking"
  | "solved";

export type Locale = "vi" | "en";

type Feature = {
  number: string;
  title: string;
  body: string;
};

type Step = {
  title: string;
  body: string;
};

export type SiteCopy = {
  meta: {
    title: string;
    description: string;
  };
  language: {
    label: string;
    vietnamese: string;
    english: string;
  };
  header: {
    homeLabel: string;
    navigationLabel: string;
    howItWorks: string;
    whyTro: string;
    backToTop: string;
    systemStatus: string;
    getTro: string;
  };
  hero: {
    practiceWindow: string;
    topic: string;
    explanationWindow: string;
    previewSteps: [string, string, string];
    voiceWindow: string;
    listening: string;
    voicePrompt: string;
    notesFolder: string;
    progressFolder: string;
    codeVariable: string;
    codeValue: string;
    tagline: string;
    description: string;
    primaryCta: string;
    secondaryCta: string;
    shortcutPrefix: string;
    shortcutSuffix: string;
    noteWindow: string;
    noteKicker: string;
    noteBody: string;
    noteAria: string;
  };
  demo: {
    label: string;
    title: string;
    statuses: Record<DemoPhase, string>;
    replay: string;
    replayLabel: string;
    address: string;
    progress: string;
    questionNumber: string;
    points: string;
    topic: string;
    question: string;
    answers: [string, string, string];
    listening: string;
    voicePrompt: string;
    thinking: string;
    understood: string;
    solutionEyebrow: string;
    solutionTitle: string;
    steps: [Step, Step, Step];
    answer: string;
    encouragement: string;
  };
  principles: {
    label: string;
    title: string;
    features: [Feature, Feature, Feature];
  };
  closing: {
    codeObject: string;
    firstLine: string;
    secondLine: string;
    cta: string;
  };
  footer: {
    statement: string;
  };
};

export const siteCopy: Record<Locale, SiteCopy> = {
  vi: {
    meta: {
      title: "Tro — Học nhẹ hơn. Tiến xa hơn.",
      description:
        "Tro là người bạn học ưu tiên tiếng Việt: hiểu màn hình, lắng nghe câu hỏi và hướng dẫn bạn từng bước.",
    },
    language: {
      label: "Chọn ngôn ngữ",
      vietnamese: "Tiếng Việt",
      english: "English",
    },
    header: {
      homeLabel: "Trang chủ Tro",
      navigationLabel: "Điều hướng chính",
      howItWorks: "cách hoạt động",
      whyTro: "vì sao chọn tro",
      backToTop: "Về đầu trang",
      systemStatus: "Trạng thái hệ thống Tro",
      getTro: "tải tro",
    },
    hero: {
      practiceWindow: "bài_tập_04",
      topic: "HÀM BẬC HAI",
      explanationWindow: "tro · lời giải thích",
      previewSteps: [
        "nhận ra dạng bài",
        "đối chiếu giá trị",
        "tự tin chọn đáp án",
      ],
      voiceWindow: "giọng nói",
      listening: "Đang lắng nghe",
      voicePrompt: "“Em chưa hiểu chỗ này…”",
      notesFolder: "ghi chú",
      progressFolder: "tiến độ",
      codeVariable: "học",
      codeValue: "dễ",
      tagline: "học nhẹ hơn. tiến xa hơn.",
      description:
        "Một người bạn học ưu tiên tiếng Việt: nhìn thấy màn hình của bạn, lắng nghe khi bạn mắc kẹt và cùng bạn đi qua từng bước tiếp theo.",
      primaryCta: "xem tro hoạt động",
      secondaryCta: "vì sao học sinh chọn tro",
      shortcutPrefix: "Nhấn",
      shortcutSuffix: "ở bất cứ đâu · macOS 14.2+",
      noteWindow: "tro nhắn",
      noteKicker: "Không chỉ là một ô đáp án.",
      noteBody: "Một gia sư có mặt ngay trên màn hình của bạn.",
      noteAria: "Nguyên tắc sản phẩm của Tro",
    },
    demo: {
      label: "Một phím tắt. Một bước tiếp theo thật rõ.",
      title: "Trợ giúp, ngay nơi bạn cần.",
      statuses: {
        idle: "Tro đã sẵn sàng",
        targeting: "Tro thấy chỗ bạn đang vướng",
        listening: "Đang nghe câu hỏi của bạn…",
        thinking: "Đang biến màn hình thành lời giải thích dễ hiểu…",
        solved: "Bạn đã có bước tiếp theo",
      },
      replay: "Xem lại",
      replayLabel: "Xem lại phần minh họa Tro",
      address: "uni.portal / giải-tích / bài-tập-04",
      progress: "4 / 10",
      questionNumber: "Câu 04",
      points: "1 điểm",
      topic: "HÀM BẬC HAI",
      question: "Đồ thị nào khớp với hàm số dưới đây?",
      answers: ["Đỉnh tại (−2, 1)", "Đỉnh tại (2, 1)", "Đỉnh tại (1, 2)"],
      listening: "Đang lắng nghe",
      voicePrompt: "“Em không biết bắt đầu từ đâu…”",
      thinking: "Đang đọc phương trình",
      understood: "Đã hiểu",
      solutionEyebrow: "Cùng làm thật đơn giản",
      solutionTitle: "Đọc tọa độ đỉnh ngay từ phương trình.",
      steps: [
        {
          title: "Nhận ra dạng bài",
          body: "Dạng đỉnh là f(x) = (x − h)² + k.",
        },
        {
          title: "Đối chiếu giá trị",
          body: "Ở đây, h = 2 và k = 1.",
        },
        {
          title: "Tự tin chọn đáp án",
          body: "Đồ thị có đỉnh tại (2, 1).",
        },
      ],
      answer: "Đáp án",
      encouragement: "Giờ hãy tự thử câu tiếp theo nhé.",
    },
    principles: {
      label: "Dành cho khoảnh khắc bạn suýt bỏ cuộc.",
      title: "Từ “em bị kẹt” đến “em làm được rồi”.",
      features: [
        {
          number: "01",
          title: "Hiểu ngữ cảnh của bạn",
          body: "Chỉ chia sẻ màn hình khi bạn yêu cầu. Tro hiểu đúng bài tập đang ở trước mắt bạn.",
        },
        {
          number: "02",
          title: "Lắng nghe tự nhiên",
          body: "Hỏi bằng tiếng Việt, tiếng Anh hoặc cả hai—như khi bạn hỏi một người bạn cùng lớp.",
        },
        {
          number: "03",
          title: "Hướng dẫn rồi lùi lại",
          body: "Nhận một lộ trình rõ ràng qua bài toán mà vẫn giữ trọn khoảnh khắc tự mình hiểu ra.",
        },
      ],
    },
    closing: {
      codeObject: "hocSinh.tuTin",
      firstLine: "Bớt mắc kẹt.",
      secondLine: "Giỏi lên mỗi ngày.",
      cta: "Trải nghiệm cách Tro giúp",
    },
    footer: {
      statement: "Ưu tiên tiếng Việt. Luôn đặt người học trước.",
    },
  },
  en: {
    meta: {
      title: "Tro — Study easier. Become your best.",
      description:
        "Tro is the Vietnamese-first desktop tutor that sees your screen, listens to your question, and guides the next step.",
    },
    language: {
      label: "Choose language",
      vietnamese: "Tiếng Việt",
      english: "English",
    },
    header: {
      homeLabel: "Tro home",
      navigationLabel: "Main navigation",
      howItWorks: "how it works",
      whyTro: "why tro",
      backToTop: "Back to top",
      systemStatus: "Tro system status",
      getTro: "get tro",
    },
    hero: {
      practiceWindow: "practice_04",
      topic: "QUADRATIC FUNCTIONS",
      explanationWindow: "tro · explanation",
      previewSteps: [
        "spot the pattern",
        "match the values",
        "choose confidently",
      ],
      voiceWindow: "voice",
      listening: "Listening",
      voicePrompt: "“I don’t understand this part…”",
      notesFolder: "notes",
      progressFolder: "progress",
      codeVariable: "study",
      codeValue: "easy",
      tagline: "study easy. become your best.",
      description:
        "A Vietnamese-first study buddy that sees your screen, listens when you are stuck, and walks you through the next step.",
      primaryCta: "see tro work",
      secondaryCta: "why students use tro",
      shortcutPrefix: "Press",
      shortcutSuffix: "anywhere · macOS 14.2+",
      noteWindow: "tro says",
      noteKicker: "Not another answer box.",
      noteBody: "A tutor that meets you on the screen.",
      noteAria: "Tro product principle",
    },
    demo: {
      label: "One shortcut. One clear next step.",
      title: "Help, right where you need it.",
      statuses: {
        idle: "Tro is ready",
        targeting: "Tro sees where you are stuck",
        listening: "Listening to your question…",
        thinking: "Turning the screen into a clear explanation…",
        solved: "You have the next step",
      },
      replay: "Replay",
      replayLabel: "Replay the Tro demonstration",
      address: "uni.portal / calculus / practice-04",
      progress: "4 of 10",
      questionNumber: "Question 04",
      points: "1 point",
      topic: "QUADRATIC FUNCTIONS",
      question: "Which graph matches the function below?",
      answers: ["Vertex at (−2, 1)", "Vertex at (2, 1)", "Vertex at (1, 2)"],
      listening: "Listening",
      voicePrompt: "“I don’t know where to start…”",
      thinking: "Reading the equation",
      understood: "Understood",
      solutionEyebrow: "Let’s make it simple",
      solutionTitle: "Read the vertex directly from the equation.",
      steps: [
        {
          title: "Spot the pattern",
          body: "Vertex form is f(x) = (x − h)² + k.",
        },
        {
          title: "Match the values",
          body: "Here, h = 2 and k = 1.",
        },
        {
          title: "Choose with confidence",
          body: "The graph has its vertex at (2, 1).",
        },
      ],
      answer: "Answer",
      encouragement: "Now try the next one yourself.",
    },
    principles: {
      label: "Made for the moment you almost give up.",
      title: "From “I’m stuck” to “I’ve got this.”",
      features: [
        {
          number: "01",
          title: "Sees your context",
          body: "Share the screen only when you ask. Tro understands the task in front of you.",
        },
        {
          number: "02",
          title: "Listens naturally",
          body: "Ask in Vietnamese, English, or both—just like you would ask a classmate.",
        },
        {
          number: "03",
          title: "Guides, then steps back",
          body: "Get a clear path through the problem without losing the learning moment.",
        },
      ],
    },
    closing: {
      codeObject: "student.confidence",
      firstLine: "Less stuck.",
      secondLine: "More becoming.",
      cta: "Experience the flow",
    },
    footer: {
      statement: "Vietnamese-first. Student-always.",
    },
  },
};
